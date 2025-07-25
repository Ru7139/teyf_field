mod course;
mod metextbook;

// ZKP
// https://zhuanlan.zhihu.com/p/14467298702
//

fn main() {}

use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
use curve25519_dalek::scalar::Scalar;
use rand::rngs::OsRng;

struct PedersenGens {
    g: RistrettoPoint,
    h: RistrettoPoint,
}

impl PedersenGens {
    fn new() -> Self {
        let mut rng = OsRng;
        let h = RistrettoPoint::random(&mut rng);

        PedersenGens {
            g: RISTRETTO_BASEPOINT_POINT,
            h,
        }
    }

    fn commit(&self, value: Scalar, blinding: Scalar) -> RistrettoPoint {
        self.g * value + self.h * blinding
    }

    fn verify(&self, commitment: RistrettoPoint, value: Scalar, blinding: Scalar) -> bool {
        let recomputed_commitment = self.commit(value, blinding);
        commitment == recomputed_commitment
    }
}
