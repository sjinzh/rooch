//# init --addresses genesis=0x1

//create account by bob self
//# run --signers genesis
script {
    use rooch_examples::counter;

    fun main(sender: &signer) {
        counter::init_(sender);
    }
}