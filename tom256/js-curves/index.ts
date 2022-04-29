import { secp256k1 } from "./instances"
import { Relation, MultiMult } from "./multimult"
import { WeierstrassPoint } from "./weier"
import { Group } from "./group"

const g1 = secp256k1.generator();
const g2 = g1.dbl();
const g4 = g2.dbl();
const g8 = g4.dbl();

let relation = new Relation(secp256k1);

/*
relation.insert(g2, secp256k1.newScalar(BigInt(10)));
relation.insert(g4, secp256k1.newScalar(BigInt(12345)));
relation.insert(g8, secp256k1.newScalar(BigInt(5500)));
*/

let multi = new MultiMult(secp256k1);

/*
let scalar1 = secp256k1.newScalar(BigInt('0xfddd45b8f6f633074edddcf1394a1c9498e6f7b5847b744adf01833f38553c01'),);
let scalar2 = secp256k1.newScalar(BigInt('0x8fdb6195754109cc23c635f41f799fd6e1f6078eb94fe0d9cde1eb80d36e5e31'));
let scalar4 = secp256k1.newScalar(BigInt('0x0c21e4f939a5d91c1473416bb936e61bd688dd91db2778f832a54cdacc207deb'));
let scalar8 = secp256k1.newScalar(BigInt('0x8fdb619575a109cc23c635f41d799fd6e1f66b8eb94fe0d9cde1eb80d36e5e31'));
*/

let scalar1 = secp256k1.newScalar(BigInt('0x83fec693ac341a0f8f3f0e6a5b18af130f3fbc2b06a00ea55743fa89e031cb5e'));
let scalar2 = secp256k1.newScalar(BigInt('0xD125353892A829607AFCB23FEBB06E84C9745F1BF040BC6D1B64672A3B9148FD'));
let scalar4 = secp256k1.newScalar(BigInt('0xF76C1FA7E623E38096A97FA0AF4D19CCE9A6D2CF62451F38D60245AED85E425F'));
let scalar8 = secp256k1.newScalar(BigInt('0x7fc351545f19ec3aecd29b4a5149a2fa56c0731cf34031e90eed16e2b78f1fa3'));



multi.insert(g1, scalar1);
multi.insert(g2, scalar2);

multi.insert(g4, scalar4);
multi.insert(g8, scalar8);

const p1 = g1.mul(scalar1);
const p2 = g2.mul(scalar2);
const p4 = g4.mul(scalar4);
const p8 = g8.mul(scalar8);

printPoint('G1', g1)
console.log("\nS1: ", scalar1.toString().slice(2));
printPoint('P1', p1)

printPoint('G2', g2)
console.log("\nS2: ", scalar2.toString().slice(2));
printPoint('P2', p2)

printPoint('G4', g4)
console.log("\nS4: ", scalar4.toString().slice(2));
printPoint('P4', p4)

printPoint('G8', g8)
console.log("\nS8: ", scalar8.toString().slice(2));
printPoint('P8', p8)

const expected = p1.add(p2.add(p4.add(p8)))
printPoint("Manual result", expected)


/*
multi.insert(g4, secp256k1.newScalar(BigInt(12345)));
multi.insert(g8, secp256k1.newScalar(BigInt(5500)));
*/

//relation.drain(multi);

let x, y = multi.evaluate()

/*
console.log(x)
console.log(x1)
console.log(x2)
console.log(x3)
console.log(x4)
console.log(x1+x2*(2**64) + x3*(2**64)*(2**64) + x4*(2**64)*(2**64)*(2**64))
*/

printPoint("Multimult result", y)



function printPoint(name: string, pt: Group.Point | WeierstrassPoint) {
    const res = pt.toAffine()
    if (res != false) {
        console.log("\n" + name + ":")
        console.log(res.x.toString(16))
        console.log(res.y.toString(16))
    }
    
}




