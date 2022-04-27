import { secp256k1 } from "./instances"
import { Relation, MultiMult } from "./multimult"

let g2 = secp256k1.generator().dbl();
let g4 = g2.dbl();
let g8 = g4.dbl();

let relation = new Relation(secp256k1);


relation.insert(g2, secp256k1.newScalar(BigInt(10)));
relation.insert(g4, secp256k1.newScalar(BigInt(12345)));
relation.insert(g8, secp256k1.newScalar(BigInt(5500)));


let multi = new MultiMult(secp256k1);

relation.drain(multi);

let x, y = multi.evaluate().toAffine();
// TODO base 16
console.log(x)
console.log(y)
