/// SendAuthorization allows the grantee to spend up to spend_limit coins from
/// the granter's account.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendAuthorization {
    #[prost(message, repeated, tag = "1")]
    pub spend_limit: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}

/// MsgSend represents a message to send coins from one account to another.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSend {
    #[prost(string, tag = "1")]
    pub from_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub to_address: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}

/// MsgSendResponse defines the Msg/Send response type.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSendResponse {}
