use crate::signature;

#[derive(Copy, Clone, Debug)]
pub enum MessageType {
    Signal,
    Error,
    Call,
    Reply,
}

#[derive(Debug)]
pub enum Base {
    Int32(i32),
    Uint32(u32),
    String(String),
    Signature(String),
    ObjectPath(String),
    Boolean(bool),
}

#[derive(Debug)]
pub enum Container {
    Array(Vec<Param>),
    Struct(Vec<Param>),
    DictEntry(Base, Box<Param>),
    Variant(Box<Variant>),
}

#[derive(Debug)]
pub struct Variant {
    pub sig: signature::Type,
    pub value: Param
}

#[derive(Debug)]
pub enum Param {
    Base(Base),
    Container(Container),
}

#[derive(Debug)]
pub struct Message {
    pub typ: MessageType,
    pub interface: Option<String>,
    pub member: Option<String>,
    pub object: Option<String>,
    pub destination: Option<String>,
    pub params: Vec<Param>,
}

impl Message {
    pub fn new(typ: MessageType, interface: Option<String>, member: Option<String>, object: Option<String>, destination: Option<String>, params: Vec<Param>) -> Message {
        Message {
            typ,
            interface,
            member,
            params,
            object,
            destination,
        }
    }
}

impl Param {
    pub fn make_signature(&self, buf: &mut String) {
        match self {
            Param::Base(b) => b.make_signature(buf),
            Param::Container(c) => c.make_signature(buf),
        }
    }
}

impl Base {
    pub fn make_signature(&self, buf: &mut String) {
        match self {
            Base::Boolean(_) => buf.push('c'),
            Base::Int32(_) => buf.push('i'),
            Base::Uint32(_) => buf.push('u'),
            Base::ObjectPath(_) => buf.push('o'),
            Base::String(_) => buf.push('s'),
            Base::Signature(_) => buf.push('g'),
        }
    }
}
impl Container {
    pub fn make_signature(&self, buf: &mut String) {
        match self {
            Container::Array(elements) => {
                buf.push('a');
                elements[0].make_signature(buf);
            },
            Container::DictEntry(key, val) => {
                buf.push('{');
                key.make_signature(buf);
                val.make_signature(buf);
                buf.push('{');
            }
            Container::Struct(elements) => {
                buf.push('(');
                for el in elements {
                    el.make_signature(buf);
                }
                buf.push(')');
            }
            Container::Variant(_) => {
                buf.push('v');
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidObjectPath,
    InvalidSignature,
    InvalidHeaderFields,
}

#[derive(Clone, Copy, Debug)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

pub enum HeaderFlags {
    NoReplyExpected,
    NoAutoStart,
    AllowInteractiveAuthorization,
}

#[derive(Debug)]
pub enum HeaderField {
    Path(String),
    Interface(String),
    Member(String),
    ErrorName(String),
    ReplySerial(u32),
    Destination(String),
    Sender(String),
    Signature(String),
    UnixFds(u32),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn validate_object_path(_op: &str) -> Result<()> {
    // TODO
    Ok(())
}
pub fn validate_signature(sig: &str) -> Result<()> {
    if signature::Type::from_str(sig).is_err() {
        Err(Error::InvalidSignature)
    } else {
        Ok(())
    }
}

pub fn validate_array(_array: &Vec<Param>) -> Result<()> {
    // TODO check that all elements have the same type
    Ok(())
}

pub fn validate_header_fields(
    msg_type: MessageType,
    header_fields: &Vec<HeaderField>,
) -> Result<()> {
    let mut have_path = false;
    let mut have_interface = false;
    let mut have_member = false;
    let mut have_errorname = false;
    let mut have_replyserial = false;
    let mut have_destination = false;
    let mut have_sender = false;
    let mut have_signature = false;
    let mut have_unixfds = false;

    for h in header_fields {
        match h {
            HeaderField::Destination(_) => {
                if have_destination {
                    return Err(Error::InvalidHeaderFields);
                }
                have_destination = true;
            }
            HeaderField::ErrorName(_) => {
                if have_errorname {
                    return Err(Error::InvalidHeaderFields);
                }
                have_errorname = true;
            }
            HeaderField::Interface(_) => {
                if have_interface {
                    return Err(Error::InvalidHeaderFields);
                }
                have_interface = true;
            }
            HeaderField::Member(_) => {
                if have_member {
                    return Err(Error::InvalidHeaderFields);
                }
                have_member = true;
            }
            HeaderField::Path(_) => {
                if have_path {
                    return Err(Error::InvalidHeaderFields);
                }
                have_path = true;
            }
            HeaderField::ReplySerial(_) => {
                if have_replyserial {
                    return Err(Error::InvalidHeaderFields);
                }
                have_replyserial = true;
            }
            HeaderField::Sender(_) => {
                if have_sender {
                    return Err(Error::InvalidHeaderFields);
                }
                have_sender = true;
            }
            HeaderField::Signature(_) => {
                if have_signature {
                    return Err(Error::InvalidHeaderFields);
                }
                have_signature = true;
            }
            HeaderField::UnixFds(_) => {
                if have_unixfds {
                    return Err(Error::InvalidHeaderFields);
                }
                have_unixfds = true;
            }
        }
    }

    let valid = match msg_type {
        MessageType::Call => {
            have_path && have_member
        }
        MessageType::Signal => {
            have_path && have_member && have_interface
        }
        MessageType::Reply => {
            have_replyserial
        }
        MessageType::Error => {
            have_errorname && have_replyserial
        }
    };
    if valid {
        Ok(())
    } else {
        Err(Error::InvalidHeaderFields)
    }
}
