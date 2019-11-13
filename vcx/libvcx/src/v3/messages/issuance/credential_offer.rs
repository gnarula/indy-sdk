use v3::messages::a2a::{MessageId, A2AMessage};
use v3::messages::issuance::CredentialPreviewData;
use v3::messages::attachment::{Attachments, Attachment, Json, AttachmentEncoding};
use v3::messages::mime_type::MimeType;
use error::{VcxError, VcxResult, VcxErrorKind};
use messages::thread::Thread;
use issuer_credential::CredentialOffer as CredentialOfferV1;
use messages::payload::PayloadKinds;
use std::convert::TryInto;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CredentialOffer {
    #[serde(rename = "@id")]
    pub id: MessageId,
    pub comment: String,
    pub credential_preview: CredentialPreviewData,
    #[serde(rename = "offers~attach")]
    pub offers_attach: Attachments,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>
}

impl CredentialOffer {
    pub fn create() -> Self {
        CredentialOffer {
            id: MessageId::new(),
            comment: String::new(),
            credential_preview: CredentialPreviewData::new(),
            offers_attach: Attachments::new(),
            thread: None,
        }
    }

    pub fn set_id(mut self, id: String) -> Self {
        self.id = MessageId(id);
        self
    }

    pub fn set_comment(mut self, comment: String) -> Self {
        self.comment = comment;
        self
    }

    pub fn set_offers_attach(mut self, credential_offer: &str) -> VcxResult<CredentialOffer> {
        let json: Json = Json::new(::serde_json::Value::String(credential_offer.to_string()), AttachmentEncoding::Base64)?;
        self.offers_attach.add(Attachment::JSON(json));
        Ok(self)
    }

    pub fn set_credential_preview_data(mut self, credential_preview: CredentialPreviewData) -> VcxResult<CredentialOffer> {
        self.credential_preview = credential_preview;
        Ok(self)
    }

    pub fn add_credential_preview_data(mut self, name: &str, value: &str, mime_type: MimeType) -> VcxResult<CredentialOffer> {
        self.credential_preview = self.credential_preview.add_value(name, value, mime_type)?;
        Ok(self)
    }

    pub fn set_thread(mut self, thread: Thread) -> Self {
        self.thread = Some(thread);
        self
    }

    pub fn to_a2a_message(&self) -> A2AMessage {
        A2AMessage::CredentialOffer(self.clone()) // TODO: THINK how to avoid clone
    }
}

impl TryInto<CredentialOffer> for CredentialOfferV1 {
    type Error = VcxError;

    fn try_into(self) -> Result<CredentialOffer, Self::Error> {
        let mut credential_preview = CredentialPreviewData::new();

        for (key, value) in self.credential_attrs {
            credential_preview = credential_preview.add_value(&key, &value.as_str().unwrap_or_default(), MimeType::Plain)?;
        }

        CredentialOffer::create()
            .set_id(self.thread_id.unwrap_or_default())
            .set_credential_preview_data(credential_preview)?
            .set_offers_attach(&self.libindy_offer)
    }
}

impl TryInto<CredentialOfferV1> for CredentialOffer {
    type Error = VcxError;

    fn try_into(self) -> Result<CredentialOfferV1, Self::Error> {
        let indy_cred_offer_json = self.offers_attach.content()?;
        let indy_cred_offer: ::serde_json::Value = ::serde_json::from_str(&indy_cred_offer_json)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize Indy Offer: {:?}", err)))?;

        let mut credential_attrs: ::serde_json::Map<String, ::serde_json::Value> = ::serde_json::Map::new();

        for attr in self.credential_preview.attributes {
            credential_attrs.insert(attr.name.clone(), ::serde_json::Value::String(attr.value.clone()));
        }

        Ok(CredentialOfferV1 {
            msg_type: PayloadKinds::CredOffer.name().to_string(),
            version: String::from("0.1"),
            to_did: String::new(),
            from_did: String::new(),
            credential_attrs,
            schema_seq_no: 0,
            claim_name: String::new(),
            claim_id: String::new(),
            msg_ref_id: None,
            cred_def_id: indy_cred_offer["cred_def_id"].as_str().map(String::from).unwrap_or_default(),
            libindy_offer: indy_cred_offer_json,
            thread_id: Some(self.id.0.clone()),
        })
    }
}