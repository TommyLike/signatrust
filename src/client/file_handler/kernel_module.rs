use std::path::PathBuf;
use super::traits::FileHandler;
use async_trait::async_trait;
use crate::util::error::Result;

use uuid::Uuid;
use std::io::Write;
use bincode::{config, Decode, Encode};
use std::collections::HashMap;
use std::os::raw::{c_uchar, c_uint};
use crate::client::cmd::options;
use crate::client::sign_identity::KeyType;
use crate::util::error::Error;


const FILE_EXTENSION: &str = "p7s";
const PKEY_ID_PKCS7: c_uchar = 2;
const MAGIC_NUMBER: &str = "~Module signature appended~\n";

// Reference https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/scripts/sign-file.c
#[derive(Encode, Decode, PartialEq, Debug)]
struct ModuleSignature {
    algo: c_uchar,       /* Public-key crypto algorithm [0] */
    hash: c_uchar,       /* Digest algorithm [0] */
    id_type: c_uchar,    /* Key identifier type [PKEY_ID_PKCS7] */
    signer_len: c_uchar, /* Length of signer's name [0] */
    key_id_len: c_uchar, /* Length of key identifier [0] */
    _pad: [c_uchar; 3],
    sig_len: c_uint, /* Length of signature data */
}

impl ModuleSignature {
    fn new(length: c_uint) -> ModuleSignature {
        ModuleSignature {
            algo: 0,
            hash: 0,
            id_type: PKEY_ID_PKCS7,
            signer_len: 0,
            key_id_len: 0,
            _pad: [0, 0, 0],
            sig_len: length,
        }
    }
}

#[derive(Clone)]
pub struct KernelModuleFileHandler {

}

impl KernelModuleFileHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate_detached_signature(&self, module: &str, signature: &[u8]) -> Result<()> {
        let mut buffer = std::fs::File::create(module)?;
        buffer.write_all(signature)?;
        Ok(())
    }

    pub fn append_inline_signature(&self, module: &str, tempfile: &str, signature: &[u8]) -> Result<()> {
        let mut signed = std::fs::File::create(tempfile)?;
        signed.write_all(&std::fs::read(module)?)?;
        signed.write_all(signature)?;
        let sig_struct = ModuleSignature::new(signature.len() as c_uint);
        signed.write_all(&bincode::encode_to_vec(
            &sig_struct,
            config::standard()
                .skip_fixed_array_length()
                .with_fixed_int_encoding()
                .with_big_endian(),
        )?)?;
        signed.write_all(MAGIC_NUMBER.as_bytes())?;
        Ok(())
    }
}

#[async_trait]
impl FileHandler for KernelModuleFileHandler {

    fn validate_options(&self, sign_options: &HashMap<String, String>) -> Result<()> {
        if let Some(key_type) = sign_options.get(options::KEY_TYPE) {
            if key_type != KeyType::X509.to_string().as_str() {
                return Err(Error::InvalidArgumentError("kernel module file only support x509 signature".to_string()))
            }
        }
        Ok(())
    }

    /* when assemble checksum signature when only create another .asc file separately */
    async fn assemble_data(&self, path: &PathBuf, data: Vec<Vec<u8>>, temp_dir: &PathBuf, sign_options: &HashMap<String, String>) -> Result<(String, String)> {
        let temp_file = temp_dir.join(Uuid::new_v4().to_string());
        //convert bytes into string
        if let Some(detached) = sign_options.get("detached") {
            if detached == "true" {
                self.generate_detached_signature(&temp_file.display().to_string(), &data[0])?;
                return Ok((temp_file.as_path().display().to_string(),
                           format!("{}.{}", path.display(), FILE_EXTENSION)))
            }
        }
        self.append_inline_signature(&path.display().to_string(), &temp_file.display().to_string(), &data[0])?;
        return Ok((temp_file.as_path().display().to_string(),
                   path.display().to_string()))

    }
}

