#![no_std]
#![no_main]

use bobcat_sdk::prelude::*;

use bobcat_entrypoints::Eip721MetadataDataSlice;

type Entry = Eip721MetadataDataSlice<1000>;

use const_hex::display as hex_display;

mod storage;

const CDN_URI_CATS: &'static str = "https://cats-cdn.superposition.so/";

fn assert_can_transfer(from: &[u8; 20], to: &[u8; 20], token_id: &U) -> [u8; 20] {
    let owner: [u8; 20] = storage::owner_of::get(token_id).into();
    assert_eq!(
        &owner,
        from,
        "from {} not owner of token id {token_id}",
        hex_display(from)
    );
    assert_ne!(to, &[0u8; 20], "to target is a zero address");
    let sender = msg_sender();
    if &sender != from
        && storage::approval::get(token_id).addr() != sender
        && !storage::approved_for_all::get(from, sender).is_some()
    {
        panic!("token id {token_id} sender is not approved");
    }
    sender
}

const TOPIC_TRANSFER: U = const_keccak256(b"Transfer(address,address,uint256)");
const TOPIC_APPROVAL: U = const_keccak256(b"Approval(address,address,uint256)");
const TOPIC_APPROVAL_FOR_ALL: U = const_keccak256(b"ApprovalForAll(address,address,bool)");

fn transfer_from(from: [u8; 20], to: [u8; 20], token_id: U) -> [u8; 20] {
    let sender = assert_can_transfer(&from, &to, &token_id);
    storage::balance::decr(from);
    storage::balance::incr(to);
    storage::owner_of::set(token_id, to);
    storage::approval::set(token_id, &U::ZERO);
    emit!(TOPIC_TRANSFER, from, to, token_id);
    sender
}

const ERC721_CB_SEL: [u8; 4] = const_keccak_sel(b"onERC721Received(address,address,uint256,bytes)");

#[derive(Debug, Clone, EvmCdSerialise)]
#[evm_selector]
enum Erc721Cb<const DATA_CAP: usize> {
    #[evm_selector("onERC721Received(address,address,uint256,bytes)")]
    OnErc721Received {
        operator: EvmCdAddress,
        from: EvmCdAddress,
        token_id: U,
        bytes: EvmCdBytes<DATA_CAP>,
    },
}

fn safe_transfer<const DATA_CAP: usize>(
    from: [u8; 20],
    to: [u8; 20],
    token_id: U,
    data: [u8; DATA_CAP],
    data_len: usize,
) {
    let sender = transfer_from(from, to, token_id);
    if addr_has_code(to) {
        let (rc, _, sel) = call_slice::<4>(
            to,
            &Erc721Cb::OnErc721Received {
                operator: EvmCdAddress::new(sender),
                from: EvmCdAddress::new(from),
                token_id,
                bytes: EvmCdBytes::<_> {
                    len: data_len,
                    bytes: data,
                },
            }
            .to_evm_array::<DATA_CAP>().unwrap(),
            &U::ZERO,
            u64::MAX,
            0,
        );
        assert!(rc, "selector callback failed");
        assert_eq!(
            ERC721_CB_SEL,
            sel,
            "returned selector not correct for callback: {}",
            hex_display(sel)
        );
    }
}

fn approve(spender: [u8; 20], id: U) {
    let owner = storage::owner_of::get(id).addr();
    let sender = msg_sender();
    assert_ne!(spender, owner, "cannot approve self");
    if sender != owner {
        assert!(
            storage::approved_for_all::get(owner, sender).is_some(),
            "not authorised"
        );
    }
    storage::approval::set(id, spender);
    emit!(TOPIC_APPROVAL, owner, spender, id)
}

fn set_approval_for_all(operator: [u8; 20], approved: bool) {
    let sender = msg_sender();
    assert_ne!(sender, operator, "cannot approval all self");
    storage::approved_for_all::set(sender, operator, approved);
    emit!(TOPIC_APPROVAL_FOR_ALL, sender, operator, approved);
}

const CAP: usize = 1000;

#[unsafe(no_mangle)]
fn user_entrypoint(len: usize) -> usize {
    match read_cd::<Entry>(len) {
        Entry::Name => {
            write_str("Superposition Supercats")
        }
        Entry::Symbol => {
            write_str("SPN CATS")
        }
        Entry::TokenUri { token_id } => {
           const URI_LEN: usize = 34;
           const TOKEN_URI_LEN: usize = URI_LEN + size_of::<U>();
           let mut buf = [0u8; TOKEN_URI_LEN];
           buf.copy_from_slice(CDN_URI_CATS.as_bytes());
           let token_str_len = token_id.str_slice_buf(&mut buf[URI_LEN..].try_into().unwrap());
           let len = URI_LEN + token_str_len;
           write_array_slice_len::<TOKEN_URI_LEN, {TOKEN_URI_LEN + 32 * 2}>(buf, len)
        }
        Entry::BalanceOf { owner } => {
            assert_ne!([0u8; 20], owner.0, "owner zero address");
            write_word(&storage::balance::get(&owner.into_array()))
        }
        Entry::OwnerOf { token_id } => {
            let owner = storage::owner_of::get(&token_id);
            assert_ne!([0u8; 20], owner.addr(), "{token_id} not minted");
            write_word(&owner)
        }
        Entry::GetApproved { token_id } => write_word(&storage::approval::get(token_id)),
        Entry::IsApprovedForAll { owner, operator } => {
            write_word(&storage::approved_for_all::get(owner.0, operator.0))
        }
        Entry::SafeTransferFrom { from, to, token_id } => {
            flush_guard(|| safe_transfer(from.into(), to.into(), token_id, [0u8; CAP], 0))
        }
        Entry::SafeTransferFromWithData {
            from,
            to,
            token_id,
            data,
        } => flush_guard(|| safe_transfer(from.into(), to.into(), token_id, data.bytes, data.len)),
        Entry::TransferFrom { from, to, token_id } => {
            flush_guard(|| transfer_from(from.into(), to.into(), token_id));
        }
        Entry::Approve { approved, token_id } => flush_guard(|| approve(approved.into(), token_id)),
        Entry::SetApprovalForAll { operator, approved } => {
            flush_guard(|| set_approval_for_all(operator.into(), approved))
        }
    };
    0
}
