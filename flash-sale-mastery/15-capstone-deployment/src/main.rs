use capstone_deployment::{DeploymentConfig, RunMode};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Determine run mode from the first argument (default: api)
    let mode = args
        .get(1)
        .and_then(|a| RunMode::from_arg(a))
        .unwrap_or(RunMode::Api);

    let config = DeploymentConfig::from_env();

    match mode {
        RunMode::Api => {
            println!(
                "[capstone-deployment] Starting API server on {}",
                config.listen_addr
            );
            println!(
                "[capstone-deployment] Redis: {} | DB: {} | Log: {}",
                config.redis_url, config.database_url, config.rust_log
            );
            // In a full implementation, this would start the axum server
            // from module 11-flash-sale-api using the config above.
            println!("[capstone-deployment] API server ready.");
            println!("[capstone-deployment] (Stub — integrate with flash-sale-api in production)");
        }
        RunMode::Worker => {
            println!("[capstone-deployment] Starting order worker");
            println!(
                "[capstone-deployment] Redis: {} | DB: {} | Log: {}",
                config.redis_url, config.database_url, config.rust_log
            );
            // In a full implementation, this would start the worker loop
            // from module 12-order-worker.
            println!("[capstone-deployment] Order worker ready.");
            println!("[capstone-deployment] (Stub — integrate with order-worker in production)");
        }
        RunMode::HealthCheck => {
            // Simple health check: verify we can parse config and exit.
            // In production, this would ping Redis and Postgres.
            println!("[capstone-deployment] Health check passed");
            std::process::exit(0);
        }
    }
}
