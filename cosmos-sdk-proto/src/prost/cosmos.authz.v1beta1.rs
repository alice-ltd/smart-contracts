/// MsgExecResponse defines the Msg/MsgExecResponse response type.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgExecResponse {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub results: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}

/// MsgExec attempts to execute the provided messages using
/// authorizations granted to the grantee. Each message should have only
/// one signer corresponding to the granter of the authorization.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgExec {
    #[prost(string, tag = "1")]
    pub grantee: ::prost::alloc::string::String,
    /// Authorization Msg requests to execute. Each msg must implement Authorization interface
    /// The x/authz will try to find a grant matching (msg.signers[0], grantee, MsgTypeURL(msg))
    /// triple and validate it.
    #[prost(message, repeated, tag = "2")]
    pub msgs: ::prost::alloc::vec::Vec<::prost_types::Any>,
}
