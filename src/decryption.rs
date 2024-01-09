use std::{
    collections::{HashMap, HashSet},
    iter::zip,
};

// Return true if the byte that originated 'c' is a space or alphabetic
fn is_space(c: u8) -> bool {
    c == 0x00 || c.is_ascii_alphabetic()
}

// XOR two byte arrays
fn xor(a: &[u8], b: &[u8]) -> Vec<u8> {
    zip(a, b).map(|(x, y)| x ^ y).collect()
}

// Return a set with the position of the spaces
fn get_space_indices(bytes: &[u8]) -> HashSet<usize> {
    bytes
        .iter()
        .enumerate()
        .filter(|(_, &c)| is_space(c))
        .map(|(i, _)| i)
        .collect()
}

// Decrypt a chunk of the key past of what we already know
fn decrypt_key_chunk(encrypted_messages: &Vec<&[u8]>) -> Vec<Option<u8>> {
    let shortest_len = encrypted_messages
        .iter()
        .map(|text| text.len())
        .min()
        .unwrap();
    let mut key: Vec<Option<u8>> = vec![None; shortest_len];
    for (i_a, msg_a) in encrypted_messages.iter().enumerate() {
        let mut counter: HashMap<usize, usize> = HashMap::new();
        for (i_b, msg_b) in encrypted_messages.iter().enumerate() {
            if i_a != i_b {
                let xor = xor(msg_a, msg_b);
                let space_indices = get_space_indices(&xor);
                space_indices.iter().for_each(|index| {
                    counter.insert(
                        *index,
                        1 + if counter.contains_key(index) {
                            counter[index]
                        } else {
                            0
                        },
                    );
                });
            }
        }
        for (i, &count) in counter.iter() {
            if count == encrypted_messages.len() - 1 {
                key[*i] = Some(msg_a[*i] ^ b' ');
            }
        }
    }
    key
}

// Decrypt the key from a list of encrypted messages
pub fn decrypt_key(encrypted_messages: &[Vec<u8>]) -> Vec<Option<u8>> {
    let mut sorted_ciphertexts: Vec<&[u8]> = encrypted_messages
        .iter()
        .map(|text| text.as_slice())
        .collect();
    sorted_ciphertexts.sort_by_key(|a| a.len());
    let mut key: Vec<Option<u8>> = Vec::new();

    let max_len = match sorted_ciphertexts.last() {
        Some(text) => text.len(),
        None => 0,
    };

    while sorted_ciphertexts.len() > 1 {
        let partial_key = decrypt_key_chunk(&sorted_ciphertexts);
        sorted_ciphertexts.remove(0);
        sorted_ciphertexts.iter_mut().for_each(|text| {
            *text = &text[partial_key.len()..];
        });
        key.extend(partial_key);
    }
    key.resize(max_len, None);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::decode_hex;

    #[test]
    fn test_dan_boneh_example() {
        let cipher_texts: Vec<Vec<u8>> = vec![
            decode_hex(concat!(
                "315c4eeaa8b5f8aaf9174145bf43e1784b8fa00dc71d885a804e5ee9fa40b16349c146fb778cdf2d",
                "3aff021dfff5b403b510d0d0455468aeb98622b137dae857553ccd8883a7bc37520e06e515d22c95",
                "4eba5025b8cc57ee59418ce7dc6bc41556bdb36bbca3e8774301fbcaa3b83b220809560987815f65",
                "286764703de0f3d524400a19b159610b11ef3e"
            ))
            .unwrap(),
            decode_hex(concat!(
                "234c02ecbbfbafa3ed18510abd11fa724fcda2018a1a8342cf064bbde548b12b07df44ba7191d960",
                "6ef4081ffde5ad46a5069d9f7f543bedb9c861bf29c7e205132eda9382b0bc2c5c4b45f919cf3a9f",
                "1cb74151f6d551f4480c82b2cb24cc5b028aa76eb7b4ab24171ab3cdadb8356f"
            ))
            .unwrap(),
            decode_hex(concat!(
                "32510ba9a7b2bba9b8005d43a304b5714cc0bb0c8a34884dd91304b8ad40b62b07df44ba6e9d8a23",
                "68e51d04e0e7b207b70b9b8261112bacb6c866a232dfe257527dc29398f5f3251a0d47e503c66e93",
                "5de81230b59b7afb5f41afa8d661cb"
            ))
            .unwrap(),
            decode_hex(concat!(
                "32510ba9aab2a8a4fd06414fb517b5605cc0aa0dc91a8908c2064ba8ad5ea06a029056f47a8ad330",
                "6ef5021eafe1ac01a81197847a5c68a1b78769a37bc8f4575432c198ccb4ef63590256e305cd3a95",
                "44ee4160ead45aef520489e7da7d835402bca670bda8eb775200b8dabbba246b130f040d8ec6447e",
                "2c767f3d30ed81ea2e4c1404e1315a1010e7229be6636aaa"
            ))
            .unwrap(),
            decode_hex(concat!(
                "3f561ba9adb4b6ebec54424ba317b564418fac0dd35f8c08d31a1fe9e24fe56808c213f17c81d960",
                "7cee021dafe1e001b21ade877a5e68bea88d61b93ac5ee0d562e8e9582f5ef375f0a4ae20ed86e93",
                "5de81230b59b73fb4302cd95d770c65b40aaa065f2a5e33a5a0bb5dcaba43722130f042f8ec85b7c",
                "2070"
            ))
            .unwrap(),
            decode_hex(concat!(
                "32510bfbacfbb9befd54415da243e1695ecabd58c519cd4bd2061bbde24eb76a19d84aba34d8de28",
                "7be84d07e7e9a30ee714979c7e1123a8bd9822a33ecaf512472e8e8f8db3f9635c1949e640c62185",
                "4eba0d79eccf52ff111284b4cc61d11902aebc66f2b2e436434eacc0aba938220b084800c2ca4e69",
                "3522643573b2c4ce35050b0cf774201f0fe52ac9f26d71b6cf61a711cc229f77ace7aa88a2f19983",
                "122b11be87a59c355d25f8e4"
            ))
            .unwrap(),
            decode_hex(concat!(
                "32510bfbacfbb9befd54415da243e1695ecabd58c519cd4bd90f1fa6ea5ba47b01c909ba7696cf60",
                "6ef40c04afe1ac0aa8148dd066592ded9f8774b529c7ea125d298e8883f5e9305f4b44f915cb2bd0",
                "5af51373fd9b4af511039fa2d96f83414aaaf261bda2e97b170fb5cce2a53e675c154c0d96815969",
                "34777e2275b381ce2e40582afe67650b13e72287ff2270abcf73bb028932836fbdecfecee0a3b894",
                "473c1bbeb6b4913a536ce4f9b13f1efff71ea313c8661dd9a4ce"
            ))
            .unwrap(),
            decode_hex(concat!(
                "315c4eeaa8b5f8bffd11155ea506b56041c6a00c8a08854dd21a4bbde54ce56801d943ba708b8a35",
                "74f40c00fff9e00fa1439fd0654327a3bfc860b92f89ee04132ecb9298f5fd2d5e4b45e40ecc3b9d",
                "59e9417df7c95bba410e9aa2ca24c5474da2f276baa3ac325918b2daada43d6712150441c2e04f65",
                "65517f317da9d3"
            ))
            .unwrap(),
            decode_hex(concat!(
                "271946f9bbb2aeadec111841a81abc300ecaa01bd8069d5cc91005e9fe4aad6e04d513e96d99de25",
                "69bc5e50eeeca709b50a8a987f4264edb6896fb537d0a716132ddc938fb0f836480e06ed0fcd6e97",
                "59f40462f9cf57f4564186a2c1778f1543efa270bda5e933421cbe88a4a52222190f471e9bd15f65",
                "2b653b7071aec59a2705081ffe72651d08f822c9ed6d76e48b63ab15d0208573a7eef027"
            ))
            .unwrap(),
            decode_hex(concat!(
                "466d06ece998b7a2fb1d464fed2ced7641ddaa3cc31c9941cf110abbf409ed39598005b3399ccfaf",
                "b61d0315fca0a314be138a9f32503bedac8067f03adbf3575c3b8edc9ba7f537530541ab0f9f3cd0",
                "4ff50d66f1d559ba520e89a2cb2a83"
            ))
            .unwrap(),
            decode_hex(concat!(
                "32510ba9babebbbefd001547a810e67149caee11d945cd7fc81a05e9f85aac650e9052ba6a8cd825",
                "7bf14d13e6f0a803b54fde9e77472dbff89d71b57bddef121336cb85ccb8f3315f4b52e301d16e9f",
                "52f904"
            ))
            .unwrap(),
        ];
        let key: Vec<u8> = decrypt_key(&cipher_texts)
            .iter()
            .map(|opt| match opt {
                Some(val) => *val,
                None => 0,
            })
            .collect();

        for byte in key.iter() {
            match *byte {
                0 => print!("_"),
                _ => print!("{:2x}", byte),
            }
        }
    }
}
