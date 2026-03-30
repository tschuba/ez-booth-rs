use super::error::{ExportError, ImportError};

pub const QR_BINARY_MAGIC: [u8; 4] = *b"EZQR";
pub const QR_BINARY_FORMAT_VERSION: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QrPayloadFormat {
    JsonV1,
    BinaryV2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryQrChunk {
    pub version: u8,
    pub index: u16,
    pub total: u16,
    pub hash: [u8; 32],
    pub data: Vec<u8>,
}

impl BinaryQrChunk {
    pub fn encode_to_bytes(&self) -> Result<Vec<u8>, ExportError> {
        let data_len = u16::try_from(self.data.len()).map_err(|_| {
            ExportError::Serialization("binary QR chunk exceeds u16 data length".to_string())
        })?;

        let mut bytes = Vec::with_capacity(43 + self.data.len());
        bytes.extend_from_slice(&QR_BINARY_MAGIC);
        bytes.push(self.version);
        bytes.extend_from_slice(&self.index.to_be_bytes());
        bytes.extend_from_slice(&self.total.to_be_bytes());
        bytes.extend_from_slice(&self.hash);
        bytes.extend_from_slice(&data_len.to_be_bytes());
        bytes.extend_from_slice(&self.data);
        Ok(bytes)
    }

    pub fn decode_from_bytes(bytes: &[u8]) -> Result<Self, ImportError> {
        if bytes.len() < 43 {
            return Err(ImportError::InvalidQrPayload(
                "binary QR payload is truncated".to_string(),
            ));
        }

        if bytes[..4] != QR_BINARY_MAGIC {
            return Err(ImportError::InvalidQrPayload(
                "binary QR payload is missing EZQR magic header".to_string(),
            ));
        }

        let version = bytes[4];
        if version != QR_BINARY_FORMAT_VERSION {
            return Err(ImportError::InvalidQrPayload(format!(
                "unsupported QR format version {version}"
            )));
        }

        let index = u16::from_be_bytes([bytes[5], bytes[6]]);
        let total = u16::from_be_bytes([bytes[7], bytes[8]]);
        let mut hash = [0_u8; 32];
        hash.copy_from_slice(&bytes[9..41]);
        let data_len = u16::from_be_bytes([bytes[41], bytes[42]]) as usize;

        if bytes.len() != 43 + data_len {
            return Err(ImportError::InvalidQrPayload(format!(
                "binary QR payload length mismatch: expected {} data bytes, found {}",
                data_len,
                bytes.len().saturating_sub(43)
            )));
        }

        Ok(Self {
            version,
            index,
            total,
            hash,
            data: bytes[43..].to_vec(),
        })
    }
}

pub fn detect_payload_format(raw: &str) -> Result<QrPayloadFormat, ImportError> {
    let trimmed = raw.trim_start();
    if trimmed.starts_with('{') {
        return Ok(QrPayloadFormat::JsonV1);
    }

    let bytes = latin1_string_to_bytes(raw)?;
    if bytes.starts_with(&QR_BINARY_MAGIC) {
        return Ok(QrPayloadFormat::BinaryV2);
    }

    Err(ImportError::InvalidQrPayload(
        "unrecognized QR payload format".to_string(),
    ))
}

pub fn latin1_string_to_bytes(raw: &str) -> Result<Vec<u8>, ImportError> {
    raw.chars()
        .map(|ch| {
            let code = ch as u32;
            u8::try_from(code).map_err(|_| {
                ImportError::InvalidQrPayload(
                    "binary QR payload contains non-Latin-1 characters".to_string(),
                )
            })
        })
        .collect()
}

pub fn bytes_to_latin1_string(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| char::from(*byte)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_chunk() -> BinaryQrChunk {
        BinaryQrChunk {
            version: QR_BINARY_FORMAT_VERSION,
            index: 2,
            total: 5,
            hash: [7_u8; 32],
            data: vec![1, 2, 3, 4, 5, 250],
        }
    }

    #[test]
    fn binary_chunk_roundtrips() {
        let chunk = sample_chunk();
        let encoded = chunk.encode_to_bytes().unwrap();
        let decoded = BinaryQrChunk::decode_from_bytes(&encoded).unwrap();
        assert_eq!(decoded, chunk);
    }

    #[test]
    fn latin1_string_roundtrips_bytes() {
        let bytes = vec![0, 1, 2, 127, 128, 200, 255];
        let encoded = bytes_to_latin1_string(&bytes);
        let decoded = latin1_string_to_bytes(&encoded).unwrap();
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn detects_json_and_binary_formats() {
        let json = r#"{\"v\":1,\"i\":0,\"t\":1,\"h\":\"00\",\"d\":\"AA==\"}"#;
        assert_eq!(
            detect_payload_format(json).unwrap(),
            QrPayloadFormat::JsonV1
        );

        let binary = sample_chunk().encode_to_bytes().unwrap();
        let payload = bytes_to_latin1_string(&binary);
        assert_eq!(
            detect_payload_format(&payload).unwrap(),
            QrPayloadFormat::BinaryV2
        );
    }

    #[test]
    fn rejects_truncated_binary_chunk() {
        let error = BinaryQrChunk::decode_from_bytes(b"EZQR\x02").unwrap_err();
        assert!(matches!(error, ImportError::InvalidQrPayload(_)));
    }
}
