include!(concat!(env!("OUT_DIR"), "/messages.rs"));

use std::sync::Arc;
use tokio::sync::Mutex;

use roslibrust::{Publish, ServiceError, Subscribe};

#[allow(dead_code)]
async fn pub_counter(ros: impl roslibrust::Ros) {
    let publisher = ros
        .advertise::<std_msgs::Int16>("example_counter")
        .await
        .unwrap();
    let mut counter = 0;
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
    loop {
        interval.tick().await;
        publisher
            .publish(&std_msgs::Int16 { data: counter })
            .await
            .unwrap();
        println!("Published {counter}");
        counter += 1;
    }
}

async fn imu_summer(ros: impl roslibrust::Ros, state: Arc<Mutex<f64>>) {
    let mut subscriber = ros
        .subscribe::<sensor_msgs::Imu>("/imu")
        .await
        .expect("Could not create subscriber!");

    loop {
        let msg = subscriber.next().await.expect("Failed to get a message!");
        *state.lock().await += msg.angular_velocity.x;
        println!(
            "Got x: {}, sum: {}",
            msg.angular_velocity.x,
            *state.lock().await
        );
    }
}

fn handle_mult_float(
    req: mult_msgs::MultFloatsRequest,
) -> Result<mult_msgs::MultFloatsResponse, ServiceError> {
    println!("Received request: {:?}", req);
    let result = req.x * req.y;
    println!("Responding with result: {}", result);
    Ok(mult_msgs::MultFloatsResponse { result })
}

async fn mult_floats(ros: impl roslibrust::Ros) {
    let _handle = ros
        .advertise_service::<mult_msgs::MultFloats, _>("/mult_floats", handle_mult_float)
        .await
        .unwrap();

    tokio::signal::ctrl_c().await.unwrap();
}

#[tokio::main]
async fn main() {
    // Create a rosbridge client we can use
    let ros = roslibrust::rosbridge::ClientHandle::new("ws://localhost:9090")
        .await
        .unwrap();

    tokio::spawn(pub_counter(ros.clone()));
    println!("Counting node has started!");
    tokio::spawn(imu_summer(ros.clone(), Arc::new(Mutex::new(0.0))));
    println!("IMU summing node has started!");
    tokio::spawn(mult_floats(ros.clone()));
    println!("Float multiplying service has started!");

    // Wait for ctrl_c
    tokio::signal::ctrl_c().await.unwrap();
}

#[cfg(test)]
mod test {
    use super::*;
    use roslibrust::{Service, ServiceProvider, Subscribe, TopicProvider};

    #[tokio::test(start_paused = true)]
    async fn test_pub_counter() {
        let ros = roslibrust::mock::MockRos::new();

        let mut subscriber = ros
            .subscribe::<std_msgs::Int16>("example_counter")
            .await
            .unwrap();

        tokio::spawn(async move { pub_counter(ros).await });

        // Confirm we get the first message
        let msg = subscriber.next().await.unwrap();
        assert_eq!(msg.data, 0);

        // Confirm second message quickly times out
        let msg =
            tokio::time::timeout(tokio::time::Duration::from_millis(10), subscriber.next()).await;
        assert!(msg.is_err());

        // Wait a bit
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        // Now get second message
        let msg = subscriber.next().await.unwrap();
        assert_eq!(msg.data, 1);
    }

    #[tokio::test(start_paused = true)]
    async fn test_imu_summer() {
        let ros = roslibrust::mock::MockRos::new();

        // Start subscriber in the background
        let shared_imu_sum = Arc::new(Mutex::new(0.0));
        {
            let shared_imu_sum = shared_imu_sum.clone();
            let ros = ros.clone();
            tokio::spawn(async move { imu_summer(ros, shared_imu_sum).await });
            tokio::time::sleep(tokio::time::Duration::from_micros(1)).await;
        }

        // Send a few messages, with their sums
        let imu_pub = ros.advertise::<sensor_msgs::Imu>("/imu").await.unwrap();
        let x_readings = vec![14.0, 67.0, -23.0];
        for x in &x_readings {
            imu_pub
                .publish(&sensor_msgs::Imu {
                    angular_velocity: geometry_msgs::Vector3 {
                        x: *x,
                        y: 0.0,
                        z: 0.0,
                    },
                    ..Default::default()
                })
                .await
                .unwrap();
            tokio::time::sleep(tokio::time::Duration::from_micros(1)).await;
        }

        // Check the result
        assert_eq!(*shared_imu_sum.lock().await, x_readings.iter().sum::<f64>());
    }

    #[tokio::test(start_paused = true)]
    async fn test_mult_service() {
        let ros = roslibrust::mock::MockRos::new();

        // Start service in the background
        let ros_clone = ros.clone();
        tokio::spawn(async move { mult_floats(ros_clone).await });
        // Wait a moment - 1us seems to be enough
        tokio::time::sleep(tokio::time::Duration::from_micros(1)).await;

        // Build service client for the service we're talking to
        let mult_client = ros
            .service_client::<mult_msgs::MultFloats>("/mult_floats")
            .await
            .unwrap();

        // Call service and confirm result is correct
        let result = mult_client
            .call(&mult_msgs::MultFloatsRequest { x: 5.0, y: 4.0 })
            .await
            .unwrap();
        assert_eq!(result.result, 20.0);

        // Wait a second, to check the service stays up
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let result = mult_client
            .call(&mult_msgs::MultFloatsRequest { x: 2.0, y: 30.0 })
            .await
            .unwrap();
        assert_eq!(result.result, 60.0);
    }
}
