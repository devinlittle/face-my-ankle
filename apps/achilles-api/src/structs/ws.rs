use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct Bootstrap<'a> {
    pub token: &'a str,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionTopic {
    WifiScan,
    DeviceStatus,
    NetworkMetrics,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum SubscriptionEvent {
    Subscribe { topics: Vec<SubscriptionTopic> },
    Unsubscribe { topics: Vec<SubscriptionTopic> },
    Query,
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionState {
    pub wifi_scan: bool,
    pub device_status: bool,
    pub network_metrics: bool,
}

impl SubscriptionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&mut self, topic: &SubscriptionTopic) {
        match topic {
            SubscriptionTopic::WifiScan => self.wifi_scan = true,
            SubscriptionTopic::DeviceStatus => self.device_status = true,
            SubscriptionTopic::NetworkMetrics => self.network_metrics = true,
        }
    }

    pub fn unsubscribe(&mut self, topic: &SubscriptionTopic) {
        match topic {
            SubscriptionTopic::WifiScan => self.wifi_scan = false,
            SubscriptionTopic::DeviceStatus => self.device_status = false,
            SubscriptionTopic::NetworkMetrics => self.network_metrics = false,
        }
    }

    pub fn is_subscribed(&self, topic: &SubscriptionTopic) -> bool {
        match topic {
            SubscriptionTopic::WifiScan => self.wifi_scan,
            SubscriptionTopic::DeviceStatus => self.device_status,
            SubscriptionTopic::NetworkMetrics => self.network_metrics,
        }
    }

    /// Returns true if the topic was newly subscribed
    pub fn subscribe_if_new(&mut self, topic: &SubscriptionTopic) -> bool {
        if self.is_subscribed(topic) {
            false
        } else {
            self.subscribe(topic);
            true
        }
    }

    /// Returns true if the topic was not subscribed
    pub fn unsubscribe_if_new(&mut self, topic: &SubscriptionTopic) -> bool {
        if !self.is_subscribed(topic) {
            false
        } else {
            self.unsubscribe(topic);
            true
        }
    }
}
