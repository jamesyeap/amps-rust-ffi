//! Subscription handling for AMPS.
//!
//! This module provides types and traits for managing AMPS subscriptions,
//! including the [`MessageHandler`] trait for processing incoming messages.
//!
//! # MessageHandler Trait
//!
//! The [`MessageHandler`] trait defines the interface for types that can
//! process AMPS messages. It is automatically implemented for closures with
//! the appropriate signature.
//!
//! # Example
//!
//! ```no_run
//! use amps_rust_ffi::Client;
//!
//! let mut client = Client::new("subscriber").unwrap();
/// client.connect("tcp://localhost:9007/amps/json").unwrap();
/// client.logon(None, 5000).unwrap();
///
/// // Using a closure as a message handler
/// client.subscribe("orders", None, |msg| {
///     println!("Received order on topic '{}': {}", msg.topic(), msg.data());
/// }).unwrap();
/// ```

use crate::message::Message;

/// A trait for types that can handle AMPS messages.
///
/// This trait is implemented for closures with the signature `FnMut(&Message) + Send`,
/// allowing you to use simple closures as message handlers. For more complex scenarios,
/// you can implement this trait for your own types.
///
/// # Thread Safety
///
/// Implementations must be `Send` because messages may be delivered on different
/// threads than the one that created the subscription.
///
/// # Example
///
/// ```no_run
/// use amps_rust_ffi::{Client, subscription::MessageHandler};
/// use amps_rust_ffi::message::Message;
///
/// struct OrderProcessor {
///     count: usize,
/// }
///
/// impl MessageHandler for OrderProcessor {
///     fn on_message(&mut self, message: &Message) {
///         self.count += 1;
///         println!("Processed {} orders", self.count);
///     }
/// }
/// ```
pub trait MessageHandler: Send {
    /// Called when a message is received.
    ///
    /// # Arguments
    ///
    /// * `message` - The received message
    fn on_message(&mut self, message: &Message);
}

impl<F> MessageHandler for F
where
    F: FnMut(&Message) + Send,
{
    fn on_message(&mut self, message: &Message) {
        (self)(message);
    }
}

/// Information about an active subscription.
///
/// This struct contains metadata about a subscription. Currently, the AMPS C++
/// client manages subscription state internally, but this type provides a
/// placeholder for future subscription management features.
#[derive(Debug, Clone)]
pub struct Subscription {
    /// The subscription ID assigned by AMPS
    pub sub_id: String,
    /// The topic or filter pattern being subscribed to
    pub topic: String,
    /// The filter expression, if any
    pub filter: Option<String>,
}

impl Subscription {
    /// Creates a new Subscription info struct.
    ///
    /// # Arguments
    ///
    /// * `sub_id` - The subscription ID
    /// * `topic` - The topic pattern
    /// * `filter` - Optional filter expression
    pub fn new(sub_id: impl Into<String>, topic: impl Into<String>, filter: Option<impl Into<String>>) -> Self {
        Subscription {
            sub_id: sub_id.into(),
            topic: topic.into(),
            filter: filter.map(Into::into),
        }
    }
}

/// Options for configuring a subscription.
#[derive(Debug, Clone, Default)]
pub struct SubscriptionOptions {
    /// Filter expression for the subscription
    pub filter: Option<String>,
    /// Additional options string
    pub options: Option<String>,
    /// Timeout in milliseconds
    pub timeout_ms: i32,
    /// Batch size for grouped delivery
    pub batch_size: i32,
    /// Top N limit for SOW queries
    pub top_n: i32,
    /// Ordering specification for SOW queries
    pub order_by: Option<String>,
}

impl SubscriptionOptions {
    /// Creates a new SubscriptionOptions with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the filter expression.
    pub fn filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }

    /// Sets additional options.
    pub fn options(mut self, options: impl Into<String>) -> Self {
        self.options = Some(options.into());
        self
    }

    /// Sets the timeout in milliseconds.
    pub fn timeout_ms(mut self, timeout_ms: i32) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Sets the batch size.
    pub fn batch_size(mut self, batch_size: i32) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// Sets the top N limit.
    pub fn top_n(mut self, top_n: i32) -> Self {
        self.top_n = top_n;
        self
    }

    /// Sets the ordering specification.
    pub fn order_by(mut self, order_by: impl Into<String>) -> Self {
        self.order_by = Some(order_by.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_new() {
        let sub = Subscription::new("sub-123", "orders", Some("price > 100"));
        assert_eq!(sub.sub_id, "sub-123");
        assert_eq!(sub.topic, "orders");
        assert_eq!(sub.filter, Some("price > 100".to_string()));
    }

    #[test]
    fn test_subscription_without_filter() {
        let sub = Subscription::new("sub-456", "trades", None::<String>);
        assert_eq!(sub.sub_id, "sub-456");
        assert_eq!(sub.topic, "trades");
        assert_eq!(sub.filter, None);
    }

    #[test]
    fn test_subscription_options_builder() {
        let opts = SubscriptionOptions::new()
            .filter("price > 100")
            .timeout_ms(5000)
            .batch_size(100);

        assert_eq!(opts.filter, Some("price > 100".to_string()));
        assert_eq!(opts.timeout_ms, 5000);
        assert_eq!(opts.batch_size, 100);
    }

    #[test]
    fn test_message_handler_closure_compiles() {
        // Test that closures implement MessageHandler
        // This test just verifies that the trait implementation compiles
        fn check_handler<T: MessageHandler>(_handler: T) {}
        
        check_handler(|_msg: &Message| {
            // Handler body
        });
    }
}
