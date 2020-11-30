use std::{
    thread::{
        self, JoinHandle,
    },
    sync::mpsc::{
        self,
        Sender, TryRecvError,
    },
    borrow::Cow
};
use reqwest::{
    header::{
        AUTHORIZATION, SEC_WEBSOCKET_PROTOCOL,
        HeaderValue,
    },
};
use tokio_tungstenite::{
    tungstenite::{
        self,
        Message, WebSocket,
        protocol::{
            frame::coding::CloseCode,
            CloseFrame
        },
        client::{
            IntoClientRequest, AutoStream,
        },
    },
};
use serde::{
    Serialize,
};

use super::*;
use crate::{
    CatenisClient,
    api::{
        NotificationEvent,
    },
    Result, Error, X_BCOT_TIMESTAMP,
    error::GenericError,
};

const NOTIFY_WS_PROTOCOL: &str = "notify.catenis.io";
const NOTIFY_WS_CHANNEL_OPEN: &str = "NOTIFICATION_CHANNEL_OPEN";

pub(crate) fn format_vec_limit<T>(v: Vec<T>, limit: usize) -> String
    where
        T: std::fmt::Debug
{
    let mut txt = format!("{:?}", &v);

    if v.len() > limit {
        txt = String::from(&txt[..txt.len() - 1]) + ", ...]";
    }

    txt
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct WsNotifyChannelAuthentication {
    x_bcot_timestamp: String,
    authorization: String,
}

enum WsNotifyChannelCommand {
    Close,
    Drop,
}

enum NotifyEventHandlerMessage {
    Drop,
    NotifyEvent(WsNotifyChannelEvent),
}

pub enum WsNotifyChannelEvent {
    Error(Error),
    Close(Option<CloseFrame<'static>>),
    Open,
    Notify(NotificationMessage)
}

#[derive(Debug)]
pub struct WsNotifyChannel<'a>{
    api_client: &'a mut CatenisClient<'a>,
    event: NotificationEvent,
    tx: Option<Sender<WsNotifyChannelCommand>>,
}

impl<'a> WsNotifyChannel<'a> {
    pub(crate) fn new(api_client: &'a mut CatenisClient<'a>, event: NotificationEvent) -> Self {
        WsNotifyChannel {
            api_client,
            event,
            tx: None,
        }
    }

    pub fn open<F>(&mut self, notify_event_handler: F) -> Result<JoinHandle<()>>
        where
            F: Fn(WsNotifyChannelEvent) + Send + 'static
    {
        let notify_event_handler = Box::new(notify_event_handler);

        // Prepare to connect to Catenis WebSocket notification service
        //  Note: this request is only used to assemble the URL for the notification service
        //      and generate the required data for authentication with the notification service.
        //      The actual request used to open a WebSocket connection is created below
        //      (from this request's URL).
        let mut auth_req = self.api_client.get_ws_request(
            "notify/ws/:event_name",
            Some(&[("event_name", self.event.to_string().as_str())])
        )?;

        self.api_client.sign_request(&mut auth_req)?;

        let ws_notify_auth_msg_json = serde_json::to_string(
            &WsNotifyChannelAuthentication {
                x_bcot_timestamp: auth_req.headers()
                    .get(X_BCOT_TIMESTAMP)
                    .unwrap_or(&HeaderValue::from_static(""))
                    .to_str()?
                    .into(),
                authorization: auth_req.headers()
                    .get(AUTHORIZATION)
                    .unwrap_or(&HeaderValue::from_static(""))
                    .to_str()?
                    .into()
            }
        )?;

        // Create request to open WebSocket connection
        let mut req = auth_req.url().as_str().into_client_request()?;

        // Add HTTP header specifying the expected WebSocket subprotocol
        req.headers_mut().insert(SEC_WEBSOCKET_PROTOCOL, HeaderValue::from_static(NOTIFY_WS_PROTOCOL));

        // Try to establish WebSocket connection
        let (mut ws, _) = tungstenite::connect(req)
            .map_err(|err| Error::new_client_error(
                Some("Failed to establish WebSocket connection"),
                Some(err)
            ))?;

        // Prepare to create thread to run WebSocket connection
        let (tx, rx) = mpsc::channel();

        // Save communication channel with WebSocket thread
        self.tx = Some(tx);

        Ok(thread::spawn(move || {
            let event_handler = notify_event_handler;

            // Create notification event handler thread
            let (h_tx, h_rx) = mpsc::channel();

            thread::spawn(move || {
                loop {
                    match h_rx.try_recv() {
                        Ok(msg) => {
                            match msg {
                                NotifyEventHandlerMessage::Drop => {
                                    // Request to exit thread. So just do it
                                    break;
                                },
                                NotifyEventHandlerMessage::NotifyEvent(event) => {
                                    // Call handler passing notification event
                                    (*event_handler)(event);
                                }
                            }
                        },
                        Err(err) => {
                            if let TryRecvError::Disconnected = err {
                                // Lost communication with parent thread. End this thread
                                break;
                            }
                        }
                    }
                }
            });

            // Send authentication message
            if let Err(err) = ws.write_message(Message::Text(ws_notify_auth_msg_json)) {
                let ctn_error = if let tungstenite::error::Error::ConnectionClosed = err {
                    // WebSocket connection has been closed
                    Error::new_client_error(
                        Some("Failed to send WebSocket notification channel authentication message; WebSocket connection closed unexpectedly"),
                        None::<GenericError>
                    )
                } else {
                    // Any other error
                    Error::new_client_error(
                        Some("Failed to send WebSocket notification channel authentication message"),
                        Some(err)
                    )
                };

                // Send error message to notification event handler thread...
                h_tx.send(
                    NotifyEventHandlerMessage::NotifyEvent(
                        WsNotifyChannelEvent::Error(ctn_error)
                    )
                ).unwrap_or(());

                // and exit current thread (requesting child thread to exit too)
                h_tx.send(NotifyEventHandlerMessage::Drop).unwrap_or(());
                return;
            }

            // Function used to receive and process command from parent thread
            let process_command = |ws2: &mut WebSocket<AutoStream>| -> Option<()> {
                match rx.try_recv() {
                    Ok(msg) => {
                        match msg {
                            WsNotifyChannelCommand::Drop => Some(()),
                            WsNotifyChannelCommand::Close => {
                                // Close WebSocket connection
                                if let Err(err) = ws2.close(Some(CloseFrame {
                                    code: CloseCode::Normal,
                                    reason: Cow::from("")
                                })) {
                                    if let tungstenite::error::Error::ConnectionClosed = err {
                                        // WebSocket connection has already been closed. Just
                                        //  indicate that current thread should exit
                                        Some(())
                                    } else {
                                        // Any other error. Send error message to notification
                                        //  event handler thread...
                                        h_tx.send(
                                            NotifyEventHandlerMessage::NotifyEvent(
                                                WsNotifyChannelEvent::Error(
                                                    Error::new_client_error(
                                                        Some("Failed to close WebSocket connection"),
                                                        Some(err)
                                                    )
                                                )
                                            )
                                        ).unwrap_or(());

                                        // and indicate that current thread should exit
                                        Some(())
                                    }
                                } else {
                                    None
                                }
                            },
                        }
                    },
                    Err(err) => {
                        match err {
                            TryRecvError::Disconnected => {
                                // Indicate that current thread should exit
                                Some(())
                            },
                            TryRecvError::Empty => None,
                        }
                    }
                }
            };

            // Check for command from parent thread before entering loop to receive
            //  data from WebSocket connection
            if let Some(()) = process_command(&mut ws) {
                // Exit current thread (requesting child thread to exit too)
                h_tx.send(NotifyEventHandlerMessage::Drop).unwrap_or(());
                return;
            }

            loop {
                match ws.read_message() {
                    Ok(msg) => {
                        match msg {
                            Message::Text(text) => {
                                // A text message was received
                                if text == NOTIFY_WS_CHANNEL_OPEN {
                                    // WebSocket notification channel open and ready to send
                                    //  notification. Send open message to notification event
                                    //  handler thread
                                    h_tx.send(
                                        NotifyEventHandlerMessage::NotifyEvent(
                                            WsNotifyChannelEvent::Open
                                        )
                                    ).unwrap_or(());
                                } else {
                                    // Parse received message
                                    match serde_json::from_str(text.as_str()) {
                                        Ok(notify_message) => {
                                            // Send notify message to notification event handler
                                            //  thread
                                            h_tx.send(
                                                NotifyEventHandlerMessage::NotifyEvent(
                                                    WsNotifyChannelEvent::Notify(notify_message)
                                                )
                                            ).unwrap_or(());
                                        },
                                        Err(_) => {
                                            // Unexpected notification message. Force closing of
                                            //  WebSocket notification channel reporting error
                                            //  condition
                                            if let Err(err) = ws.close(Some(CloseFrame {
                                                code: CloseCode::Library(4000),
                                                reason: Cow::from(format!("Unexpected notification message received: {}", text))
                                            })) {
                                                if let tungstenite::error::Error::ConnectionClosed = err {
                                                    // WebSocket connection has already been closed. Just exit
                                                    //  current thread (requesting child thread to exit too)
                                                    h_tx.send(NotifyEventHandlerMessage::Drop).unwrap_or(());
                                                    return;
                                                } else {
                                                    // Any other error. Send error message to notification
                                                    //  event handler thread...
                                                    h_tx.send(
                                                        NotifyEventHandlerMessage::NotifyEvent(
                                                            WsNotifyChannelEvent::Error(
                                                                Error::new_client_error(
                                                                    Some("Failed to close WebSocket connection"),
                                                                    Some(err)
                                                                )
                                                            )
                                                        )
                                                    ).unwrap_or(());

                                                    // and exit current thread (requesting child thread to exit too)
                                                    h_tx.send(NotifyEventHandlerMessage::Drop).unwrap_or(());
                                                    return;
                                                }
                                            }
                                        },
                                    }
                                }
                            },
                            Message::Binary(bin) => {
                                // A binary message was received. This is unexpected, so
                                //  force closing of WebSocket notification channel reporting
                                //  the error condition
                                if let Err(err) = ws.close(Some(CloseFrame {
                                    code: CloseCode::Unsupported,
                                    reason: Cow::from(format!("Unexpected binary message received: {}", format_vec_limit(bin, 20)))
                                })) {
                                    if let tungstenite::error::Error::ConnectionClosed = err {
                                        // WebSocket connection has already been closed. Just exit
                                        //  current thread (requesting child thread to exit too)
                                        h_tx.send(NotifyEventHandlerMessage::Drop).unwrap_or(());
                                        return;
                                    } else {
                                        // Any other error. Send error message to notification
                                        //  event handler thread...
                                        h_tx.send(
                                            NotifyEventHandlerMessage::NotifyEvent(
                                                WsNotifyChannelEvent::Error(
                                                    Error::new_client_error(
                                                        Some("Failed to close WebSocket connection"),
                                                        Some(err)
                                                    )
                                                )
                                            )
                                        ).unwrap_or(());

                                        // and exit current thread (requesting child thread to exit too)
                                        h_tx.send(NotifyEventHandlerMessage::Drop).unwrap_or(());
                                        return;
                                    }
                                }
                            },
                            Message::Ping(_) | Message::Pong(_) => (),
                            Message::Close(close_info) => {
                                // WebSocket connection is being closed. Send close message
                                //  to notification event handler thread...
                                h_tx.send(
                                    NotifyEventHandlerMessage::NotifyEvent(
                                        WsNotifyChannelEvent::Close(close_info)
                                    )
                                ).unwrap_or(());

                                // and continue precessing normally until receiving confirmation
                                //  (via Error::ConnectionClosed) that WebSocket connection has
                                //  been closed
                            }
                        }
                    },
                    Err(err) => {
                        if let tungstenite::error::Error::ConnectionClosed = err {
                            // WebSocket connection has been closed
                        } else {
                            // Any other error. Send error message to notification event handler thread...
                            h_tx.send(
                                NotifyEventHandlerMessage::NotifyEvent(
                                    WsNotifyChannelEvent::Error(
                                        Error::new_client_error(
                                            Some("Failed to send WebSocket notification channel authentication message"),
                                            Some(err)
                                        )
                                    )
                                )
                            ).unwrap_or(());
                        };

                        // Exit current thread (requesting child thread to exit too)
                        h_tx.send(NotifyEventHandlerMessage::Drop).unwrap_or(());
                        return;
                    }
                }

                // Check for command from parent thread
                if let Some(()) = process_command(&mut ws) {
                    // Exit current thread (requesting child thread to exit too)
                    h_tx.send(NotifyEventHandlerMessage::Drop).unwrap_or(());
                    return;
                }
            }
        }))
    }

    pub fn close(&mut self) {
        if let Some(tx) = &self.tx {
            // Send command to notification event handling thread to close WebSocket
            //  notification channel
            tx.send(WsNotifyChannelCommand::Close).unwrap_or(());
        }
    }
}

impl<'a> Drop for WsNotifyChannel<'a> {
    fn drop(&mut self) {
        if let Some(tx) = &self.tx {
            // Send command to notification event handling thread to stop it
            tx.send(WsNotifyChannelCommand::Drop).unwrap_or(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_serialize_ws_auth() {
        let st = WsNotifyChannelAuthentication {
            x_bcot_timestamp: String::from("YYYY-MM-ddTHH:mm:ssZ"),
            authorization: String::from("blablabla"),
        };

        let json = serde_json::to_string(&st).unwrap();

        println!(">>>>>> WebSocket notification channel authentication json: {}", json);
    }

    #[test]
    fn it_deserialize_notify_msg() {
        let msg = r#"{
            "messageId": "mjfaklreuiewjkd",
            "to": {
              "deviceId": "dreuwnvnvlshfhsa"
            },
            "readDate": "2020-11-26T15:35:00Z"
        }"#;

        let notify_msg: NotificationMessage = serde_json::from_str(msg).unwrap();

        println!(">>>>>> Deserialized notification message: {:?}", notify_msg);
    }

    #[test]
    fn it_format_vector() {
        let v: Vec<u8> = vec![0x01, 0x02, 0x03];

        /*let txt = format!("{:?}", &v);
        let txt = String::from(&txt[..txt.len()-1]);*/
        let txt = format_vec_limit(v, 2);

        println!(">>>>>> Formatted vector: {}", txt);
    }

    #[test]
    fn it_opens_ws_notify_channel() {
        use crate::*;

        let api_access_secret = "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3";
        let device_id = "drc3XdxNtzoucpw9xiRp";

        let mut ctn_client = CatenisClient::new_with_options(
            api_access_secret,
            device_id,
            &[
                ClientOptions::Host("localhost:3000"),
                ClientOptions::Secure(false),
                ClientOptions::UseCompression(false)
            ],
        ).unwrap();

        let mut notify_channel = ctn_client.new_ws_notify_channel(NotificationEvent::NewMsgReceived);

        let notify_thread = notify_channel.open(|event: WsNotifyChannelEvent| {
            match event {
                WsNotifyChannelEvent::Error(err) => {
                    println!(">>>>>> WebSocket Notification Channel: Error event: {:?}", err);
                },
                WsNotifyChannelEvent::Open => {
                    println!(">>>>>> WebSocket Notification Channel: Open event");
                },
                WsNotifyChannelEvent::Close(close_info) => {
                    println!(">>>>>> WebSocket Notification Channel: Close event: {:?}", close_info);
                },
                WsNotifyChannelEvent::Notify(notify_msg) => {
                    println!(">>>>>> WebSocket Notification Channel: Notify event: {:?}", notify_msg);
                },
            }
        }).unwrap();

        // Wait for events to be received
        thread::sleep(std::time::Duration::from_secs(60));

        // Close WebSocket notification channel
        notify_channel.close();

        // Wait for notification thread to end
        notify_thread.join().unwrap_or(());
    }
}