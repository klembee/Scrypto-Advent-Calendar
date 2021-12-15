# Day 15 - Vacine Passport as NFTs
Today, NFTs were integrated into Scrypto as part of the Alexandria release. We are going to make a vacine passport represented as a NFT.

## How to test

### Setup the VacinePassport component
1. Reset your environment: `resim reset`
2. Create the default account: `resim new-account`
3. Build and publish the blueprints: `resim publish .`. Take note of the returned package address
4. Instantiate a new VacinePassport component: `resim call-function [package_address] VacinePassport new`. The second ResourceDef is the address of the passport NFT resource definition. Remember it.
5. Create a new empty passport: `resim call-method [component_address] get_new_passport`.

### Setup the party component
6. Instantiate a new ChristmasParty component: `resim call-function [package_address] ChristmasParty new [passport_nft_address]`
7. Try to enter the party: `resim call-method [party_component_address] enter_party 1,[passport_nft_address]`. You should get a message saying you are not authorized to get in.

### Geting in the party
8. Take a vacine: `resim call-method [component_address] get_vacine 1,[passport_nft_address]`
9. Display the data of the vacines you have taken: `resim call-method [passport_component_address] display_vacine_data 1,[passport_nft_address]`
10. Try to enter the party again: `resim call-method [party_component_address] enter_party 1,[passport_nft_address]`. You should now be able to enter the party !