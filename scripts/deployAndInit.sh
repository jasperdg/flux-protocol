# near deploy --wasmFile $1 --accountId $2.flux-dev --masterAccount flux-dev --initFunction new --initArgs '{"owner": "flux-dev", "fun_token_account_id": "'$3'.flux-dev"}'
near deploy --wasmFile $1 --accountId $2.flux-dev --masterAccount flux-dev --initFunction init --initArgs '{"owner": "flux-dev", "fun_token_account_id": "'$3'.flux-dev"}'
