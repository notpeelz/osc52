#[derive(Debug, thiserror::Error)]
pub enum Base64EncodeError {
    #[error("input too large to encode")]
    InputTooLarge,
}

pub fn encode(input: &[u8]) -> Result<String, Base64EncodeError> {
    let encoded_len = input
        .len()
        .checked_add(2)
        .and_then(|n| n.checked_div(3))
        .and_then(|n| n.checked_mul(4))
        .ok_or(Base64EncodeError::InputTooLarge)?;

    let mut output = String::with_capacity(encoded_len);

    for chunk in input.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);

        // Combine into 24 bits
        let triple = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);

        // Extract 6-bit indices
        let i0 = ((triple >> 18) & 0x3F) as usize;
        let i1 = ((triple >> 12) & 0x3F) as usize;
        let i2 = ((triple >> 6) & 0x3F) as usize;
        let i3 = (triple & 0x3F) as usize;

        const TABLE: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        output.push(TABLE[i0] as char);
        output.push(TABLE[i1] as char);

        match chunk.len() {
            3 => {
                output.push(TABLE[i2] as char);
                output.push(TABLE[i3] as char);
            }
            2 => {
                output.push(TABLE[i2] as char);
                output.push('=');
            }
            1 => {
                output.push('=');
                output.push('=');
            }
            _ => unreachable!(),
        }
    }

    Ok(output)
}

#[derive(Debug, thiserror::Error)]
pub enum Base64DecodeError {
    #[error("input length must be a multiple of 4")]
    InvalidLength,

    #[error("invalid base64 character: '{0}'")]
    InvalidCharacter(char),

    #[error("invalid padding")]
    InvalidPadding,
}

pub fn decode(input: &str) -> Result<Vec<u8>, Base64DecodeError> {
    fn val(c: u8) -> Result<Option<u8>, Base64DecodeError> {
        match c {
            b'A'..=b'Z' => Ok(Some(c - b'A')),
            b'a'..=b'z' => Ok(Some(c - b'a' + 26)),
            b'0'..=b'9' => Ok(Some(c - b'0' + 52)),
            b'+' => Ok(Some(62)),
            b'/' => Ok(Some(63)),
            b'=' => Ok(None),
            _ => Err(Base64DecodeError::InvalidCharacter(c as char)),
        }
    }

    let bytes = input.as_bytes();

    if bytes.len() % 4 != 0 {
        return Err(Base64DecodeError::InvalidLength);
    }

    let mut output = Vec::with_capacity(bytes.len() / 4 * 3);

    for (i, chunk) in bytes.chunks(4).enumerate() {
        let is_last = i == (bytes.len() / 4) - 1;
        let v0 = val(chunk[0])?;
        let v1 = val(chunk[1])?;
        let v2 = val(chunk[2])?;
        let v3 = val(chunk[3])?;

        let v0 = v0.ok_or(Base64DecodeError::InvalidPadding)?;
        let v1 = v1.ok_or(Base64DecodeError::InvalidPadding)?;
        output.push((v0 << 2) | (v1 >> 4));

        match (v2, v3) {
            (Some(v2), Some(v3)) => {
                output.push((v1 << 4) | (v2 >> 2));
                output.push((v2 << 6) | v3);
            }
            (Some(v2), None) => {
                if !is_last {
                    return Err(Base64DecodeError::InvalidPadding);
                }
                output.push((v1 << 4) | (v2 >> 2));
            }
            (None, None) => {
                if !is_last {
                    return Err(Base64DecodeError::InvalidPadding);
                }
            }
            (None, Some(_)) => {
                return Err(Base64DecodeError::InvalidPadding);
            }
        }
    }

    Ok(output)
}
