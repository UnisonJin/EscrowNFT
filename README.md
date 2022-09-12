# Escrow contract

## Create Escrow (Source User)

The user send his NFT to Escorw contract with send message (which is basic function of cw721 base contract). 

-Options
 
 1.Set the expiration time for escrow.
 
 2.Set the recipient for the NFT.
 
 3.Set the price of NFT.

## Accept the NFT (Recipient)

  -Check if the sent money from recipeint to the escrow contract is the same as the amount of fund which the    
   source user set
 
  -Validate the expiration time. 

## Withdraw NFT (Source User)

 If the expiration is finished, the source user can withdraw his NFT.
 