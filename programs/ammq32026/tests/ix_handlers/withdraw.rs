use anchor_lang::{
    solana_program::instruction::Instruction,
InstructionData, ToAccountMetas,    
};
use anchor_spl::associated_token;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use litesvm_token::spl_token::ID as TOKEN_PROGRAM_ID;
pub fn create_withdraw_ix(
    payer: &Keypair,
    mint_x: Pubkey,
    mint_y: Pubkey,
    mint_lp: Pubkey,
    config: Pubkey,
    vault_x: Pubkey,
    vault_y: Pubkey,
) -> Instruction {
    let user = payer.pubkey();
    let user_x = associated_token::get_associated_token_address(&user, &mint_x);
    let user_y = associated_token::get_associated_token_address(&user, &mint_y);
    let user_lp = associated_token::get_associated_token_address(&user, &mint_lp);
    Instruction::new_with_bytes(
        ammq32026::id(),
        &ammq32026::instruction::Withdraw{
            amount: 50_000_000,
            min_x: 0,
            min_y: 0,
        }
        .data(),
        ammq32026::accounts::Withdraw{
            user,
            mint_x,
            mint_y,
            config,
            mint_lp,
            vault_x,
            vault_y,
            user_x,
            user_y,
            user_lp,
            token_program: TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}