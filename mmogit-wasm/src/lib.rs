use wasm_bindgen::prelude::*;
use ed25519_dalek::{SigningKey, Signature, Signer};
use serde::{Serialize, Deserialize};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Serialize, Deserialize)]
pub struct VisitorMessage {
    pub timestamp: String,
    pub message: String,
    pub pubkey: String,
    pub signature: String,
    pub user_agent: Option<String>,
}

#[wasm_bindgen]
pub struct VisitorBook {
    signing_key: SigningKey,
    pubkey: String,
}

#[wasm_bindgen]
impl VisitorBook {
    /// Create a new ephemeral visitor identity
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<VisitorBook, JsValue> {
        // Generate random key for this visitor
        let mut rng = rand::thread_rng();
        let signing_key = SigningKey::generate(&mut rng);
        let pubkey = hex::encode(signing_key.verifying_key().to_bytes());
        
        console_log!("Created ephemeral visitor identity: {}", &pubkey[..8]);
        
        Ok(VisitorBook {
            signing_key,
            pubkey,
        })
    }
    
    /// Sign a visitor message
    #[wasm_bindgen]
    pub fn sign_visit(&self, message: &str, user_agent: Option<String>) -> Result<String, JsValue> {
        let timestamp = js_sys::Date::new_0().to_iso_string().as_string().unwrap();
        
        // Create message to sign
        let msg = VisitorMessage {
            timestamp: timestamp.clone(),
            message: message.to_string(),
            pubkey: self.pubkey.clone(),
            signature: String::new(), // Will fill after signing
            user_agent,
        };
        
        // Sign the message content
        let sign_data = format!("{}{}{}", msg.timestamp, msg.message, msg.pubkey);
        let signature: Signature = self.signing_key.sign(sign_data.as_bytes());
        
        // Create final message with signature
        let signed_msg = VisitorMessage {
            signature: hex::encode(signature.to_bytes()),
            ..msg
        };
        
        console_log!("Visitor {} signed: {}", &self.pubkey[..8], message);
        
        serde_json::to_string(&signed_msg)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    
    /// Get the visitor's public key
    #[wasm_bindgen]
    pub fn get_pubkey(&self) -> String {
        self.pubkey.clone()
    }
    
    /// Create a git commit message for this visit
    #[wasm_bindgen]
    pub fn create_commit_message(&self, visitor_number: u32) -> String {
        format!(
            "🌟 Visitor #{} looked up with us\n\nPubkey: {}...{}\nTime: {}\n\nSigned with mmogit-wasm",
            visitor_number,
            &self.pubkey[..8],
            &self.pubkey[self.pubkey.len()-8..],
            js_sys::Date::new_0().to_iso_string().as_string().unwrap()
        )
    }
}

#[wasm_bindgen]
pub fn init() {
    console_log!("🚀 mmogit-wasm initialized - Sovereign visitor book ready!");
}