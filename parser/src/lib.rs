use nom::{
    bytes::complete::{tag, take_until, take_while},
    character::complete::newline,
    error::Error,
    branch::alt,
};
use thiserror::Error;

#[derive(Error, Debug)]
enum Err {
    #[error("conversion")]
    BytesCharsParsing(String),
    #[error("missing tag")]
    Tag(String), // Maybe use enum to specify section 
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Sign{
    #[default]
    Signed,
    Unsigned
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Order {
    #[default]
    LittleEndian,
    // BigEndian,
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Definition {
    name: String,
    id: String,
    bytes: u32,
    sender: String,
    signed: Sign,
    // scale: u64,
    // offset: u64,
    // order: Order,
}
// BO_ 500 IO_DEBUG: 4 IO
//  SG_ IO_DEBUG_test_unsigned : 0|8@1+ (1,0) [0|0] "" DBG

// Observations:
//
// The message name is IO_DEBUG and MID is 500 (decimal), and the length is 4 bytes (though we only need 1 for 8-bit signal)
// The sender is IO
// 0|8: The unsigned signal starts at bit position 0, and the size of this signal is 8
// (1,0): The scale and offset (discussed later)
// [0|0]: Min and Max is not defined (discussed later)
// "": There are no units (it could be, for instance "inches")
// @1+: Defines that the signal is little-endian, and unsigned: Never change this!

fn parse_message<'input>(inp: &'input [u8]) -> Result<(&'input [u8], Definition), Err> {
    // First line
    // B0_
    let (tail, _b0) = tag("BO_".as_bytes())(inp)
        .map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag("BO_ not found".to_string()))?;
    // Space
    let (tail, _space) =
        tag(" ")(tail).map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag("space after BO_".into()))?;
    // Message ID
    let (tail, msg_id) = take_while(|u| (u as char).is_ascii_digit())(tail)
        .map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag("messageID".into()))?;
    // Space
    let (tail, _space) = tag(" ")(tail)
        .map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag("space after message ID".into()))?;
    // Message name
    let (tail, name) =
        take_until(":")(tail).map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag(":".into()))?;
    // :
    let (tail, _colon) =
        tag(":")(tail).map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag(":".into()))?;
    // Space
    let (tail, _space) =
        tag(" ")(tail).map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag("space after colon".into()))?;
    // msg length?
    let (tail, bytes_chars) = take_until(" ")(tail)
        .map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag("bytes chars".into()))?;
    let bytes_string = String::from_utf8(bytes_chars.to_vec()).unwrap();
    let bytes: u32 = bytes_string
        .parse()
        .map_err(|_| Err::BytesCharsParsing(format!("{:?} not digits", &bytes_chars)))?;
    // Space
    let (tail, _space) =
        tag(" ")(tail).map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag("space after colon".into()))?;
    // Sender
    let (tail, sender_bytes) = take_while(|u| (u as char) != '\n')(tail)
        .map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag("messageID".into()))?;
    // newline
    let (tail, _newline) =
        newline(tail).map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag("space after colon".into()))?;
    // Second line
    // space
    let (tail, _space) =
        tag(" ")(tail).map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag("space after BO_".into()))?;
    // SG_
    let (tail, _sg_) =
        tag("SG_")(tail).map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag("space after BO_".into()))?;
    // Space
    let (tail, _space) =
        tag(" ")(tail).map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag("space after BO_".into()))?;
    // msg_id2
    let (tail, _msg_id2) =
        tag(name)(tail).map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag(":".into()))?;
    // _ Underscore
    let (tail, msg_id2) =
        tag("_")(tail).map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag("space after 107".into()))?;
    // test_signed
    let (tail, signed_test) =
        alt((tag("test_signed"), tag("test_unsigned")))(tail).map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag("space after 111".into()))?;
    // Space Colon Space
    let (tail, msg_id2) =
        tag(" : ")(tail).map_err(|_: nom::Err<Error<&[u8]>>| Err::Tag("space after 107".into()))?;
    
    let name = String::from_utf8(name.to_vec()).unwrap();
    Ok((tail, Definition {
        name,
        id: String::from_utf8(msg_id.to_vec()).unwrap(),
        bytes,
        sender: String::from_utf8(sender_bytes.to_vec()).unwrap(),
        signed: if std::str::from_utf8(signed_test) == Ok("test_signed") { Sign::Signed } else { Sign::Unsigned}
    }))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tests_parse_message() {
        let msg = r#"BO_ 500 IO_DEBUG: 4 IO
 SG_ IO_DEBUG_test_unsigned : 0|8@1+ (1,0) [0|0] "" DBG"#
            .as_bytes();
        assert_eq!(
            parse_message(msg).unwrap()[0],
            Definition {
                name: "IO_DEBUG".to_string(),
                id: "500".to_string(),
                sender: "IO".to_string(),
                signed: Sign::Unsigned,
                bytes: 4,
            }
        );
    }
}
