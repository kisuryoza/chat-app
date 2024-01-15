//! These types are provided through API to a client

use std::borrow::Cow;

use crate::prelude::*;
use chat_macros::{Get, New};

#[derive(New, Get, Debug)]
pub struct Entity<'a> {
    timestamp: i64,
    kind: Box<EventKind<'a>>,
}

impl<'a> Entity<'a> {
    pub fn expect_handshake(&'a self) -> Result<&Handshake> {
        match *self.kind {
            EventKind::Handshake(ref inner) => Ok(inner),
            _ => Err(Error::decode("Bad event structure")),
        }
    }

    pub fn expect_registration_request(&'a self) -> Result<&RegistrationRequest<'a>> {
        match *self.kind {
            EventKind::Registration(Registration::Request(ref inner)) => Ok(inner),
            _ => Err(Error::decode("Bad event structure")),
        }
    }
    pub fn expect_registration_response(&'a self) -> Result<&RegistrationResponse> {
        match *self.kind {
            EventKind::Registration(Registration::Response(ref inner)) => Ok(inner),
            _ => Err(Error::decode("Bad event structure")),
        }
    }

    pub fn expect_authentication_request(&'a self) -> Result<&AuthenticationRequest<'a>> {
        match *self.kind {
            EventKind::Authentication(Authentication::Request(ref inner)) => Ok(inner),
            _ => Err(Error::decode("Bad event structure")),
        }
    }
    pub fn expect_authentication_response(&'a self) -> Result<&AuthenticationResponse> {
        match *self.kind {
            EventKind::Authentication(Authentication::Response(ref inner)) => Ok(inner),
            _ => Err(Error::decode("Bad event structure")),
        }
    }

    pub fn expect_message(&'a self) -> Result<&Message<'_>> {
        match *self.kind {
            EventKind::Message(ref inner) => Ok(inner),
            _ => Err(Error::decode("Bad event structure")),
        }
    }
}

#[derive(Debug)]
pub enum EventKind<'a> {
    Handshake(Handshake),
    Registration(Registration<'a>),
    Authentication(Authentication<'a>),
    Message(Message<'a>),
}

#[derive(New, Get, Debug)]
pub struct Handshake {
    pub_key: PublicKey,
}

///////////////////////////////////////////////////////////////////////////////
// Registration
#[derive(Debug)]
pub enum Registration<'a> {
    Request(RegistrationRequest<'a>),
    Response(RegistrationResponse),
}

#[derive(New, Get, Debug)]
pub struct RegistrationRequest<'a> {
    username: Cow<'a, str>,
    password: Cow<'a, str>,
}

#[derive(New, Get, Debug)]
pub struct RegistrationResponse {
    status: RegistrationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegistrationStatus {
    Success,
    UserExists,
}

impl std::fmt::Display for RegistrationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "Success"),
            Self::UserExists => write!(f, "User exists"),
        }
    }
}

impl TryFrom<i32> for RegistrationStatus {
    type Error = crate::error::Error;
    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            x if x == Self::Success as i32 => Ok(Self::Success),
            x if x == Self::UserExists as i32 => Ok(Self::UserExists),
            _ => Err(crate::error::Error::decode("Bad event structure")),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Authentication
#[derive(Debug)]
pub enum Authentication<'a> {
    Request(AuthenticationRequest<'a>),
    Response(AuthenticationResponse),
}

#[derive(New, Get, Debug)]
pub struct AuthenticationRequest<'a> {
    username: Cow<'a, str>,
    password: Cow<'a, str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuthenticationStatus {
    Success,
    UserDoesNotExist,
    WrongPassword,
}

impl std::fmt::Display for AuthenticationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "Success"),
            Self::UserDoesNotExist => write!(f, "User does not exist"),
            Self::WrongPassword => write!(f, "Wrong password"),
        }
    }
}

impl TryFrom<i32> for AuthenticationStatus {
    type Error = crate::error::Error;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            x if x == Self::Success as i32 => Ok(Self::Success),
            x if x == Self::UserDoesNotExist as i32 => Ok(Self::UserDoesNotExist),
            x if x == Self::WrongPassword as i32 => Ok(Self::WrongPassword),
            _ => Err(crate::error::Error::decode("Bad event structure")),
        }
    }
}

#[derive(New, Get, Debug)]
pub struct AuthenticationResponse {
    status: AuthenticationStatus,
}

///////////////////////////////////////////////////////////////////////////////
// Message
#[derive(New, Get, Debug)]
pub struct Message<'a> {
    sender: Cow<'a, str>,
    text: Cow<'a, str>,
}
