use std::{
    thread::{
        self, JoinHandle,
    },
    sync::mpsc::{
        self,
        SyncSender, TryRecvError,
    },
    borrow::Cow
};
use reqwest::{
    header::{
        AUTHORIZATION, SEC_WEBSOCKET_PROTOCOL,
        HeaderValue,
    },
};
use tungstenite::{
    self,
    Message,
    protocol::{
        frame::coding::CloseCode,
    },
    client::{
        IntoClientRequest, AutoStream,
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

pub use tungstenite::protocol::CloseFrame;

pub(crate) const NOTIFY_WS_PROTOCOL: &str = "notify.catenis.io";
pub(crate) const NOTIFY_WS_CHANNEL_OPEN: &str = "NOTIFICATION_CHANNEL_OPEN";

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
pub(crate) struct WsNotifyChannelAuthentication {
    pub(crate) x_bcot_timestamp: String,
    pub(crate) authorization: String,
}

pub(crate) enum WsNotifyChannelCommand {
    Close,
    Drop,
}

pub(crate) enum NotifyEventHandlerMessage {
    Drop,
    NotifyEvent(WsNotifyChannelEvent),
}

#[derive(Debug)]
pub enum WsNotifyChannelEvent {
    Error(Error),
    Close(Option<CloseFrame<'static>>),
    Open,
    Notify(NotificationMessage)
}

#[derive(Debug, Clone)]
pub struct WsNotifyChannel{
    pub(crate) api_client: CatenisClient,
    pub(crate) event: NotificationEvent,
    tx: Option<SyncSender<WsNotifyChannelCommand>>,
}

impl WsNotifyChannel {
    pub(crate) fn new(api_client: &CatenisClient, event: NotificationEvent) -> Self {
        WsNotifyChannel {
            api_client: api_client.clone(),
            event,
            tx: None,
        }
    }

    pub fn open<F>(&mut self, notify_event_handler: F) -> Result<JoinHandle<()>>
        where
            F: Fn(WsNotifyChannelEvent) + Send + 'static
    {
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

        // Set read timeout for WebSocket connection
        match ws.get_ref() {
            AutoStream::Plain(stream) =>  stream,
            AutoStream::Tls(tls_stream) => tls_stream.get_ref(),
        }.set_read_timeout(Some(std::time::Duration::from_millis(500)))
            .map_err(|err| Error::new_client_error(
                Some("Failed to set read timeout for WebSocket connection"),
                Some(err)
            ))?;

        // Prepare to create thread to run WebSocket connection
        let (tx, rx) = mpsc::sync_channel(128);

        // Save communication channel with WebSocket thread
        self.tx = Some(tx);

        Ok(thread::spawn(move || {
            // Create notification event handler thread
            let (h_tx, h_rx) = mpsc::channel();

            thread::spawn(move || {
                loop {
                    match h_rx.recv() {
                        Ok(msg) => {
                            match msg {
                                NotifyEventHandlerMessage::Drop => {
                                    // Request to exit thread. So just do it
                                    break;
                                },
                                NotifyEventHandlerMessage::NotifyEvent(event) => {
                                    // Call handler passing notification event
                                    notify_event_handler(event);
                                }
                            }
                        },
                        Err(_) => {
                            // Lost communication with parent thread. End this thread
                            break;
                        },
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

            loop {
                // Receive data from WebSocket connection
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
                        let mut err_to_report = None;
                        let mut exit = false;

                        match &err {
                            tungstenite::error::Error::Io(io_err) => {
                                match io_err.kind() {
                                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut => {
                                        // Timeout reading data from WebSocket connection. Just
                                        //  continue processing
                                    },
                                    _ => {
                                        // Any other I/O error. Indicate that error should be
                                        //  reported and thread exited
                                        err_to_report = Some(err);
                                        exit = true;
                                    }
                                }
                            },
                            tungstenite::error::Error::ConnectionClosed => {
                                // WebSocket connection has been closed. Indicate that
                                //  thread should be exited
                                exit = true;
                            },
                            _ => {
                                // Any other error. Indicate that error should be
                                //  reported and thread exited
                                err_to_report = Some(err);
                                exit = true;
                            }
                        }

                        if let Some(err) = err_to_report {
                            // Send error message to notification event
                            //  handler thread
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
                        }

                        if exit {
                            // Exit current thread (requesting child thread to exit too)
                            h_tx.send(NotifyEventHandlerMessage::Drop).unwrap_or(());
                            return;
                        }
                    }
                }

                // Check for command from main thread
                match rx.try_recv() {
                    Ok(msg) => {
                        match msg {
                            WsNotifyChannelCommand::Drop => {
                                // Exit current thread (requesting child thread to exit too)
                                h_tx.send(NotifyEventHandlerMessage::Drop).unwrap_or(());
                                return;
                            },
                            WsNotifyChannelCommand::Close => {
                                // Close WebSocket connection
                                if let Err(err) = ws.close(Some(CloseFrame {
                                    code: CloseCode::Normal,
                                    reason: Cow::from("")
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
                    },
                    Err(err) => {
                        match err {
                            TryRecvError::Disconnected => {
                                // Lost communication with main thread. Exit current thread
                                //  (requesting child thread to exit too)
                                h_tx.send(NotifyEventHandlerMessage::Drop).unwrap_or(());
                                return;
                            },
                            TryRecvError::Empty => {
                                // No data to be received now. Just continue processing
                            }
                        }
                    },
                }
            }
        }))
    }

    pub fn close(&self) {
        if let Some(tx) = &self.tx {
            // Send command to notification event handler thread to close WebSocket
            //  notification channel
            tx.send(WsNotifyChannelCommand::Close).unwrap_or(());
        }
    }
}

impl Drop for WsNotifyChannel {
    fn drop(&mut self) {
        if let Some(tx) = &self.tx {
            // Send command to notification event handler thread to stop it
            tx.send(WsNotifyChannelCommand::Drop).unwrap_or(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_serialize_ws_notify_channel_authentication() {
        let ws_notify_channel_authentication = WsNotifyChannelAuthentication {
            x_bcot_timestamp: String::from("20201210T203848Z"),
            authorization: String::from("CTN1-HMAC-SHA256 Credential=drc3XdxNtzoucpw9xiRp/20201210/ctn1_request, Signature=7c8a878788b0bf6ddcc38f47a590ed6b261cb18a0261fefb42f9db1ee2fcb866"),
        };

        let json = serde_json::to_string(&ws_notify_channel_authentication).unwrap();

        assert_eq!(json, r#"{"x-bcot-timestamp":"20201210T203848Z","authorization":"CTN1-HMAC-SHA256 Credential=drc3XdxNtzoucpw9xiRp/20201210/ctn1_request, Signature=7c8a878788b0bf6ddcc38f47a590ed6b261cb18a0261fefb42f9db1ee2fcb866"}"#);
    }

    #[test]
    fn it_process_ws_notify_channel_events() {
        use std::sync::{Arc, Mutex};
        use crate::*;

        let ctn_client = CatenisClient::new_with_options(
            Some((
                "drc3XdxNtzoucpw9xiRp",
                "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3",
            ).into()),
            &[
                ClientOptions::Host("localhost:3000"),
                ClientOptions::Secure(false),
                ClientOptions::UseCompression(false)
            ],
        ).unwrap();

        // Open WebSocket notification channel closing it after first notify message is received
        let notify_channel = Arc::new(Mutex::new(
            ctn_client.new_ws_notify_channel(NotificationEvent::NewMsgReceived)
        ));
        let notify_channel_2 = notify_channel.clone();

        let notify_thread = notify_channel.lock().unwrap()
            // Note: we need to access a reference of notify_channel inside the notify_event_handler
            //  closure. That's why we need to wrap it around Arc<Mutex<>> (see above)
            .open(move |event: WsNotifyChannelEvent| {
                let notify_channel = notify_channel_2.lock().unwrap();

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
                        notify_channel.close();
                    },
                }
            }).unwrap();

        // Set up timeout to close WebSocket notification channel if no notify message
        //  is received within a given period of time
        let notify_channel_3 = notify_channel.clone();

        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_secs(30));

            notify_channel_3.lock().unwrap()
                .close();
        });

        // Wait for notification thread to end
        notify_thread.join().unwrap();
    }
}