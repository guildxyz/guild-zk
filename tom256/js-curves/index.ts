import { secp256k1 } from "./instances"
import { Relation, MultiMult } from "./multimult"
import { WeierstrassPoint } from "./weier"
import { Group } from "./group"


function printPoint(name: string, pt: Group.Point | WeierstrassPoint) {
    const res = pt.toAffine()
    if (res != false) {
        console.log("\n" + name + ":")
        console.log(res.x.toString(16))
        console.log(res.y.toString(16))
    }
    
}

// Generating testing data for Relation
const g1 = secp256k1.generator();
const g2 = g1.dbl();
const g4 = g2.dbl();
const g8 = g4.dbl();

let relation = new Relation(secp256k1);

let scalar1 = secp256k1.newScalar(BigInt('0x83fec693ac341a0f8f3f0e6a5b18af130f3fbc2b06a00ea55743fa89e031cb5e'));
let scalar2 = secp256k1.newScalar(BigInt('0xD125353892A829607AFCB23FEBB06E84C9745F1BF040BC6D1B64672A3B9148FD'));
let scalar4 = secp256k1.newScalar(BigInt('0xF76C1FA7E623E38096A97FA0AF4D19CCE9A6D2CF62451F38D60245AED85E425F'));
let randomizer = secp256k1.newScalar(BigInt('0x7fc351545f19ec3aecd29b4a5149a2fa56c0731cf34031e90eed16e2b78f1fa3'));

relation.insert(g1, scalar1);
relation.insert(g2, scalar2);
relation.insert(g4, scalar4);

let multi = new MultiMult(secp256k1);
relation.drain_with_randomizer(multi, randomizer);

let x, y = multi.evaluate()

printPoint("Result", y)






