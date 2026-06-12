use order_worker::p01_consumer::OrderConsumer;
use order_worker::p02_order_processor::OrderProcessor;
use order_worker::p03_payment_handler::PaymentHandler;
use order_worker::p04_notification_handler::NotificationHandler;
use order_worker::p05_dead_letter::DeadLetterQueue;

use deadpool_redis::Config;
use tracing::{error, info, warn};

const STREAM_KEY: &str = "orders:stream";
const GROUP_NAME: &str = "order-workers";
const CONSUMER_NAME: &str = "worker-1";
const DLQ_STREAM: &str = "orders:dlq";

/// Entry point for the order worker.
///
/// Connects to Redis, creates a consumer group, and enters the main processing
/// loop. Each iteration reads a batch of events from the stream, processes them
/// through the order processor, payment handler, and notification handler. Failed
/// events are pushed to the dead letter queue.
///
/// # Environment Variables
///
/// - `REDIS_URL` -- Redis connection string (default: `redis://127.0.0.1:6379`)
/// - `CONSUMER_NAME` -- Unique consumer name (default: `worker-1`)
/// - `RUST_LOG` -- Log level (default: `info`)
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let consumer_name =
        std::env::var("CONSUMER_NAME").unwrap_or_else(|_| CONSUMER_NAME.to_string());

    info!("Order worker starting...");
    info!("Redis: {} | Consumer: {}", redis_url, consumer_name);

    let cfg = Config::from_url(&redis_url);
    let pool = cfg
        .builder()
        .expect("Failed to configure Redis pool")
        .build()
        .expect("Failed to create Redis pool");

    let mut consumer = OrderConsumer::new(pool.clone(), STREAM_KEY, GROUP_NAME, &consumer_name)
        .await?;

    let processor = OrderProcessor::new();
    let payment = PaymentHandler::new();
    let notifications = NotificationHandler::new();
    let dlq = DeadLetterQueue::new(pool.clone(), DLQ_STREAM).await?;

    info!("Order worker ready. Entering processing loop...");

    loop {
        match consumer.consume().await {
            Ok(events) => {
                for event in events {
                    let event_id = event.stream_id.clone();

                    // Step 1: Process the event into an order
                    match processor.process(event) {
                        Ok(order) => {
                            info!(
                                order_id = %order.id,
                                product_id = %order.product_id,
                                "Order created"
                            );

                            // Step 2: Attempt payment
                            match payment.process_payment(&order).await {
                                Ok(result) => {
                                    info!(
                                        order_id = %order.id,
                                        payment = ?result,
                                        "Payment processed"
                                    );

                                    // Step 3: Send confirmation (best-effort)
                                    if let Err(e) =
                                        notifications.send_confirmation(&order).await
                                    {
                                        warn!(
                                            order_id = %order.id,
                                            error = %e,
                                            "Notification failed (non-fatal)"
                                        );
                                    }

                                    // Acknowledge the stream message
                                    if let Err(e) = consumer.acknowledge(&event_id).await {
                                        error!(
                                            event_id = %event_id,
                                            error = %e,
                                            "Failed to acknowledge event"
                                        );
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        order_id = %order.id,
                                        error = %e,
                                        "Payment failed"
                                    );
                                    // Send failure notification (best-effort)
                                    let _ = notifications.send_failure(&order).await;
                                    // Push to DLQ for retry
                                    dlq.push(
                                        order_worker::p01_consumer::OrderEvent {
                                            stream_id: event_id.clone(),
                                            order_id: order.id.to_string(),
                                            product_id: order.product_id.clone(),
                                            account_id: order.account_id.clone(),
                                            voucher_code: order.voucher_code.clone(),
                                        },
                                        &e.to_string(),
                                    )
                                    .await?;
                                    // Still acknowledge -- we've moved it to DLQ
                                    let _ = consumer.acknowledge(&event_id).await;
                                }
                            }
                        }
                        Err(e) => {
                            warn!(
                                event_id = %event_id,
                                error = %e,
                                "Event processing failed"
                            );
                            // Acknowledge invalid events to avoid reprocessing
                            let _ = consumer.acknowledge(&event_id).await;
                        }
                    }
                }
            }
            Err(e) => {
                error!(error = %e, "Failed to consume events");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }

        // Brief pause to avoid tight-looping when the stream is empty
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
