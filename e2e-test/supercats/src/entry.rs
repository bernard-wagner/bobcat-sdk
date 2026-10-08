use bobcat_sdk::cd::{EvmCdAddress, EvmCdBytes, EvmCdDeserialise, EvmCdSerialise, U};

#[derive(Debug, Clone, PartialEq, EvmCdSerialise, EvmCdDeserialise)]
#[evm_selector]
pub enum Eip721MetadataDataSlice<const MAX_DATA: usize> {
    Mint {
        owner: EvmCdAddress,
        token_id: U,
    },
    BalanceOf {
        owner: EvmCdAddress,
    },
    OwnerOf {
        token_id: U,
    },
    SafeTransferFrom {
        from: EvmCdAddress,
        to: EvmCdAddress,
        token_id: U,
    },
    #[evm_selector("safeTransferFrom(address,address,uint256,bytes)")]
    SafeTransferFromWithData {
        from: EvmCdAddress,
        to: EvmCdAddress,
        token_id: U,
        data: EvmCdBytes<MAX_DATA>,
    },
    TransferFrom {
        from: EvmCdAddress,
        to: EvmCdAddress,
        token_id: U,
    },
    Approve {
        approved: EvmCdAddress,
        token_id: U,
    },
    SetApprovalForAll {
        operator: EvmCdAddress,
        approved: bool,
    },
    GetApproved {
        token_id: U,
    },
    IsApprovedForAll {
        owner: EvmCdAddress,
        operator: EvmCdAddress,
    },
    Name,
    Symbol,
    #[evm_selector("tokenURI(uint256)")]
    TokenUri {
        token_id: U,
    },
}
