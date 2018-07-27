//! Utility methods to be used when working with RPC services.

const FNV1A_INIT: u32 = 0x811c9dc5;
const FNV1A_PRIME: u32 = 0x01000193;

/// Hashes the provided string like with FNV-1a (32-bit variant).
pub fn hash_service_name<S: AsRef<str>>(name: S) -> u32 {
    let mut hash = FNV1A_INIT;
    for byte in name.as_ref().bytes() {
        hash = hash ^ byte as u32;
        hash = hash.overflowing_mul(FNV1A_PRIME).0;
    }

    return hash;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hash_verification() {
        let test_one = "bnet.protocol.authentication.AuthenticationServer";
        let hash_one = hash_service_name(test_one);
        assert_eq!(233634817, hash_one);

        let test_two = "bnet.protocol.channel.ChannelSubscriber";
        let hash_two = hash_service_name(test_two);
        assert_eq!(3213656212, hash_two);

        let response_test = "bnet.protocol.ResponseService";
        let hash_response = hash_service_name(response_test);
        println!("{:} - {:}", hash_response, response_test);
    }
}
