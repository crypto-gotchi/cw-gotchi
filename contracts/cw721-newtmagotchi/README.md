# CW721 NewtMagotchi Contract

A Newtmagotchi is a living nft that you have to feed to keep alive. Anyone can feed a Newtmagotchi, but its only true to one owner. The T will have a lastFed property. This property will reflect its health state. If a user feeds it, it has to pay a little bit of tokens, but the longer you wait, the more expensive itll be.

This Contract is an extension on the CW721-base contract with the following Extensions:

### Execute Extension:

`Feed`: will feed the newtmagotchi and reset the lastFed property. The cost of feeding is based on the number of days since it has been fed. if it hasnt been fed for MAX_DAYS_WITHOUT_FOOD days, it will Die and never be feedable again.

 d