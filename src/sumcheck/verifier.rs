use crate::polynomials::{
    multilinear_poly::MultilinearPolynomial, univariate_poly::UnivariatePolynomial,
};
use ark_ff::PrimeField;

pub struct Verifier<F: PrimeField> {
    pub initial_poly: MultilinearPolynomial<F>,
    pub challenges: Vec<F>,
    pub last_round_sum: F,
}

impl<F: PrimeField> Verifier<F> {
    pub fn new(poly: MultilinearPolynomial<F>, claimed_sum: F) -> Self {
        Verifier {
            initial_poly: poly,
            challenges: vec![],
            last_round_sum: claimed_sum,
        }
    }

    pub fn sample_challenge(&mut self) {
        let mut rng = ark_std::test_rng();
        let random_challenge = F::rand(&mut rng);
        self.challenges.push(random_challenge);
    }

    pub fn verify(&mut self, poly: UnivariatePolynomial<F>) -> bool {
        let verifier_check = poly.evaluate(F::zero()) + poly.evaluate(F::one());

        if verifier_check != self.last_round_sum {
            return false;
        }

        self.sample_challenge();

        self.last_round_sum = poly.evaluate(self.challenges[self.challenges.len() - 1]);

        if self.challenges.len() == self.initial_poly.terms[0].vars.len() {
            return self.last_round_sum
                == self
                    .initial_poly
                    .evaluate(self.challenges.clone().into_iter().enumerate().collect());
        }

        true
    }
}
