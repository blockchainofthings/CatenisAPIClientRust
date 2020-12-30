use std::{
    borrow::Cow,
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
        Message,
        protocol::{
            frame::coding::CloseCode,
            CloseFrame,
        },
        client::{
            IntoClientRequest,
        },
    },
};
use futures_util::{
    SinkExt, StreamExt,
};
use tokio::{
    task::{
        JoinHandle,
    },
    sync::{
        mpsc::{
            self,
            error::TryRecvError,
        }
    },
};

use crate::{
    api::{
        NotificationEvent,
    },
    Result, Error, X_BCOT_TIMESTAMP,
    error::GenericError,
    notification::*,
    async_impl::{
        client::CatenisClient,
    }
};

#[derive(Debug, Clone)]
pub struct WsNotifyChannel{
    pub(crate) api_client: CatenisClient,
    pub(crate) event: NotificationEvent,
    pub(crate) tx: Option<mpsc::Sender<WsNotifyChannelCommand>>,
}

impl WsNotifyChannel {
    pub(crate) fn new(api_client: &CatenisClient, event: NotificationEvent) -> Self {
        WsNotifyChannel {
            api_client: api_client.clone(),
            event,
            tx: None,
        }
    }

    pub async fn open<F>(&mut self, notify_event_handler: F) -> Result<JoinHandle<()>>
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
        let (mut ws, _) = tokio_tungstenite::connect_async(req)
            .await
            .map_err(|err| Error::new_client_error(
                Some("Failed to establish WebSocket connection"),
                Some(err)
            ))?;

        // Prepare to async task to run WebSocket connection
        let (tx, mut rx) = mpsc::channel(128);

        // Save communication channel with WebSocket async task
        self.tx = Some(tx);

        Ok(tokio::spawn(async move {
            // Create notification event handler async task
            let (mut h_tx, mut h_rx) = mpsc::channel(1024);

            tokio::spawn(async move {
                loop {
                    match h_rx.recv().await {
                        Some(msg) => {
                            match msg {
                                NotifyEventHandlerMessage::Drop => {
                                    // Request to exit async task. So just do it
                                    break;
                                },
                                NotifyEventHandlerMessage::NotifyEvent(event) => {
                                    // Call handler passing notification event
                                    notify_event_handler(event);
                                }
                            }
                        },
                        None => {
                            // Lost communication with parent async task. End this task
                            break;
                        },
                    }
                }
            });

            // Send authentication message
            if let Err(err) = ws.send(Message::Text(ws_notify_auth_msg_json)).await {
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

                // Send error message to notification event handler async task...
                h_tx.send(
                    NotifyEventHandlerMessage::NotifyEvent(
                        WsNotifyChannelEvent::Error(ctn_error)
                    )
                ).await.unwrap_or(());

                // and exit current async task (requesting child async task to exit too)
                h_tx.send(NotifyEventHandlerMessage::Drop).await.unwrap_or(());
                return;
            }

            loop {
                // Receive data from WebSocket connection
                match tokio::time::timeout(std::time::Duration::from_millis(500),ws.next()).await {
                    Ok(next_result) => {
                        match next_result {
                            Some(result) => {
                                match result {
                                    Ok(msg) => {
                                        match msg {
                                            Message::Text(text) => {
                                                // A text message was received
                                                if text == NOTIFY_WS_CHANNEL_OPEN {
                                                    // WebSocket notification channel open and ready to send
                                                    //  notification. Send open message to notification event
                                                    //  handler async task
                                                    h_tx.send(
                                                        NotifyEventHandlerMessage::NotifyEvent(
                                                            WsNotifyChannelEvent::Open
                                                        )
                                                    ).await.unwrap_or(());
                                                } else {
                                                    // Parse received message
                                                    match serde_json::from_str(text.as_str()) {
                                                        Ok(notify_message) => {
                                                            // Send notify message to notification event handler
                                                            //  async task
                                                            h_tx.send(
                                                                NotifyEventHandlerMessage::NotifyEvent(
                                                                    WsNotifyChannelEvent::Notify(notify_message)
                                                                )
                                                            ).await.unwrap_or(());
                                                        },
                                                        Err(_) => {
                                                            // Unexpected notification message. Force closing of
                                                            //  WebSocket notification channel reporting error
                                                            //  condition
                                                            if let Err(err) = ws.close(Some(CloseFrame {
                                                                code: CloseCode::Library(4000),
                                                                reason: Cow::from(format!("Unexpected notification message received: {}", text))
                                                            })).await {
                                                                if let tungstenite::error::Error::ConnectionClosed = err {
                                                                    // WebSocket connection has already been closed. Just exit
                                                                    //  current async task (requesting child async task to exit too)
                                                                    h_tx.send(NotifyEventHandlerMessage::Drop).await.unwrap_or(());
                                                                    return;
                                                                } else {
                                                                    // Any other error. Send error message to notification
                                                                    //  event handler async task...
                                                                    h_tx.send(
                                                                        NotifyEventHandlerMessage::NotifyEvent(
                                                                            WsNotifyChannelEvent::Error(
                                                                                Error::new_client_error(
                                                                                    Some("Failed to close WebSocket connection"),
                                                                                    Some(err)
                                                                                )
                                                                            )
                                                                        )
                                                                    ).await.unwrap_or(());

                                                                    // and exit current async task (requesting child async task to exit too)
                                                                    h_tx.send(NotifyEventHandlerMessage::Drop).await.unwrap_or(());
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
                                                })).await {
                                                    if let tungstenite::error::Error::ConnectionClosed = err {
                                                        // WebSocket connection has already been closed. Just exit
                                                        //  current async task (requesting child async task to exit too)
                                                        h_tx.send(NotifyEventHandlerMessage::Drop).await.unwrap_or(());
                                                        return;
                                                    } else {
                                                        // Any other error. Send error message to notification
                                                        //  event handler async task...
                                                        h_tx.send(
                                                            NotifyEventHandlerMessage::NotifyEvent(
                                                                WsNotifyChannelEvent::Error(
                                                                    Error::new_client_error(
                                                                        Some("Failed to close WebSocket connection"),
                                                                        Some(err)
                                                                    )
                                                                )
                                                            )
                                                        ).await.unwrap_or(());

                                                        // and exit current async task (requesting child async task to exit too)
                                                        h_tx.send(NotifyEventHandlerMessage::Drop).await.unwrap_or(());
                                                        return;
                                                    }
                                                }
                                            },
                                            Message::Ping(_) | Message::Pong(_) => (),
                                            Message::Close(close_info) => {
                                                // WebSocket connection is being closed. Send close message
                                                //  to notification event handler async task...
                                                h_tx.send(
                                                    NotifyEventHandlerMessage::NotifyEvent(
                                                        WsNotifyChannelEvent::Close(close_info)
                                                    )
                                                ).await.unwrap_or(());

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
                                            // Any other error. Send error message to notification event
                                            //  handler async task
                                            h_tx.send(
                                                NotifyEventHandlerMessage::NotifyEvent(
                                                    WsNotifyChannelEvent::Error(
                                                        Error::new_client_error(
                                                            Some("Failed to send WebSocket notification channel authentication message"),
                                                            Some(err)
                                                        )
                                                    )
                                                )
                                            ).await.unwrap_or(());
                                        };

                                        // Exit current async task (requesting child async task to exit too)
                                        h_tx.send(NotifyEventHandlerMessage::Drop).await.unwrap_or(());
                                        return;
                                    }
                                }
                            },
                            None => {
                                // Assume that WebSocket connection has been closed, and
                                //  just exit current async task (requesting child async task to exit too)
                                h_tx.send(NotifyEventHandlerMessage::Drop).await.unwrap_or(());
                                return;
                            },
                        }
                    },
                    Err(_) => {
                        // Timeout reading data from WebSocket connection. Just
                        //  continue processing
                    },
                }

                // Check for command from parent thread
                match rx.try_recv() {
                    Ok(msg) => {
                        match msg {
                            WsNotifyChannelCommand::Drop => {
                                // Exit current async task (requesting child async task to exit too)
                                h_tx.send(NotifyEventHandlerMessage::Drop).await.unwrap_or(());
                                return;
                            },
                            WsNotifyChannelCommand::Close => {
                                // Close WebSocket connection
                                if let Err(err) = ws.close(Some(CloseFrame {
                                    code: CloseCode::Normal,
                                    reason: Cow::from("")
                                })).await {
                                    if let tungstenite::error::Error::ConnectionClosed = err {
                                        // WebSocket connection has already been closed. Just exit
                                        //  current async task (requesting child async task to exit too)
                                        h_tx.send(NotifyEventHandlerMessage::Drop).await.unwrap_or(());
                                        return;
                                    } else {
                                        // Any other error. Send error message to notification
                                        //  event handler async task...
                                        h_tx.send(
                                            NotifyEventHandlerMessage::NotifyEvent(
                                                WsNotifyChannelEvent::Error(
                                                    Error::new_client_error(
                                                        Some("Failed to close WebSocket connection"),
                                                        Some(err)
                                                    )
                                                )
                                            )
                                        ).await.unwrap_or(());

                                        // and exit current async task (requesting child async task to exit too)
                                        h_tx.send(NotifyEventHandlerMessage::Drop).await.unwrap_or(());
                                        return;
                                    }
                                }
                            },
                        }
                    },
                    Err(err) => {
                        match err {
                            TryRecvError::Closed => {
                                // Lost communication with main thread. Exit current async task
                                //  (requesting child async task to exit too)
                                h_tx.send(NotifyEventHandlerMessage::Drop).await.unwrap_or(());
                                return;
                            },
                            TryRecvError::Empty => {
                                // No data to be received now. Just continue processing
                            },
                        }
                    },
                }
            }
        }))
    }

    pub async fn close(&mut self) {
        if let Some(tx) = &mut self.tx {
            // Send command to notification event handler async task to close
            //  WebSocket notification channel
            tx.send(WsNotifyChannelCommand::Close).await.unwrap_or(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_process_ws_notify_channel_events() {
        use std::sync::{Arc, Mutex};
        use crate::{
            async_impl::CatenisClient,
            ClientOptions,
        };

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

        let notify_task;

        {
            notify_task = notify_channel.lock().unwrap()
                // Note: we need to access a reference of notify_channel inside the notify_event_handler
                //  closure. That's why we need to wrap it around Arc<Mutex<>> (see above)
                .open(move |event: WsNotifyChannelEvent| {
                    // Note: clone (the dereferenced) notify_channel so it can be moved into
                    //  spawned async task
                    let mut notify_channel = (&*notify_channel_2.lock().unwrap()).clone();

                    tokio::spawn(async move {
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
                                notify_channel.close().await;
                            },
                        }
                    });
                }).await.unwrap();
        }

        // Set up timeout to close WebSocket notification channel if no notify message
        //  is received within a given period of time
        // Note: clone (a new dereferenced reference of) notify_channel so it can be moved
        //  into spawned thread
        let mut notify_channel_3 = (&*notify_channel.clone().lock().unwrap()).clone();

        tokio::spawn(async move {
            tokio::time::delay_for(std::time::Duration::from_secs(30)).await;

            notify_channel_3.close().await;
        });

        // Wait for notification task to end
        notify_task.await.unwrap();
    }
}