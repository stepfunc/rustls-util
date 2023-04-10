use rustls::client::ServerCertVerified;
use rustls::server::ClientCertVerified;
use rustls::{Certificate, CertificateError, DistinguishedName, Error, ServerName};
use rx509::der::ASNError;
use std::time::SystemTime;

/// Verifier that can used as a client or server verifier based on a pre-shared peer certificate
pub struct SelfSignedVerifier {
    /// expected certificate
    expected_peer_cert: Certificate,
    /// pre-parsed validity
    validity: rx509::x509::Validity,
}

impl SelfSignedVerifier {
    /// Create a verifier specifying the expected peer certificate.
    ///
    /// This method performs a light parsing of the certificate using [rx509](https://crates.io/crates/rx509)
    /// to extract the Validity (not before, not after) time interval for the certificate so that
    /// can be later used during validation. An error is returned if the certificate cannot be parsed.
    pub fn create(expected: Certificate) -> Result<Self, impl std::error::Error> {
        let parsed = rx509::x509::Certificate::parse(&expected.0)?;

        let validity = parsed.tbs_certificate.value.validity;

        Ok::<Self, ASNError>(Self {
            expected_peer_cert: expected,
            validity,
        })
    }

    fn verify_peer(
        &self,
        end_entity: &Certificate,
        intermediates: &[Certificate],
        now: SystemTime,
    ) -> Result<(), Error> {
        // Check that no intermediate certificates are present
        if !intermediates.is_empty() {
            let msg = format!(
                "client sent {} intermediate certificates, expected none",
                intermediates.len()
            );
            return Err(Error::General(msg));
        }

        // Check that presented certificate matches byte-for-byte the expected certificate
        if end_entity != &self.expected_peer_cert {
            return Err(Error::InvalidCertificate(CertificateError::UnknownIssuer));
        }

        let now = now
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| Error::FailedToGetCurrentTime)?;
        let now = rx509::der::UtcTime::from_seconds_since_epoch(now.as_secs());

        if !self.validity.is_valid(now) {
            return Err(Error::InvalidCertificate(CertificateError::Expired));
        }

        // We do not validate DNS name. Providing the exact same certificate is sufficient.

        Ok(())
    }
}

impl rustls::server::ClientCertVerifier for SelfSignedVerifier {
    fn client_auth_root_subjects(&self) -> &[DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        end_entity: &Certificate,
        intermediates: &[Certificate],
        now: SystemTime,
    ) -> Result<ClientCertVerified, Error> {
        self.verify_peer(end_entity, intermediates, now)?;
        Ok(ClientCertVerified::assertion())
    }
}

impl rustls::client::ServerCertVerifier for SelfSignedVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &Certificate,
        intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        now: SystemTime,
    ) -> Result<ServerCertVerified, Error> {
        self.verify_peer(end_entity, intermediates, now)?;
        Ok(ServerCertVerified::assertion())
    }
}
