use std::{
    collections::BTreeMap,
    ops::{Add, Mul},
};

use ark_ff::{BigInteger, PrimeField};
use serde::{Deserialize, Serialize};

use crate::utils::get_binary_string;

// Multilinear Monomial representation where
// coefficient = Coefficient of the monomial
// vars = variables vector where index represents variable
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MultilinearMonomial<F: PrimeField> {
    pub coefficient: F,
    pub vars: Vec<bool>,
}

// Multilinear Polynomial representation
// terms = mMonomial terms f the polynomial
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MultilinearPolynomial<F: PrimeField> {
    pub terms: Vec<MultilinearMonomial<F>>,
}

/// Convert a selector to an index
fn selector_to_index(selector: &[bool]) -> usize {
    let mut sum = 0;
    let mut adder = 1;

    for i in 0..selector.len() {
        if selector[i] {
            sum += adder;
        }
        adder *= 2;
    }

    sum
}

// Returns the multilinear polynomial representing a bit value
fn check_bit<F: PrimeField>(bit: usize) -> MultilinearPolynomial<F> {
    let mut res = MultilinearPolynomial::new(vec![]);
    // The check: xiwi + (1 - xi)(1 - wi)
    if bit == 0 {
        res = MultilinearPolynomial::new(vec![
            MultilinearMonomial::new(F::one(), vec![false]),
            MultilinearMonomial::new(F::one().neg(), vec![true]),
        ]);
    };
    if bit == 1 {
        res = MultilinearPolynomial::new(vec![MultilinearMonomial::new(F::one(), vec![true])]);
    }

    res
}

// Multilinear monomial implementation
impl<F: PrimeField> MultilinearMonomial<F> {
    // Creates new multilinear monomial
    pub fn new(coefficient: F, vars: Vec<bool>) -> Self {
        Self { coefficient, vars }
    }

    // Adds two multilinear monomial
    pub fn add(self, rhs: MultilinearMonomial<F>) -> MultilinearPolynomial<F> {
        let mut res = MultilinearPolynomial::<F>::new(vec![]);

        if self.vars == rhs.vars {
            res.terms.push(MultilinearMonomial::new(
                self.coefficient + rhs.coefficient,
                self.vars,
            ));
        } else {
            res.terms.push(self);
            res.terms.push(rhs);
        }

        res
    }

    // Multiply two multilinear monomial
    pub fn multiply(&mut self, rhs: &mut MultilinearMonomial<F>) -> MultilinearMonomial<F> {
        let mut new_vars = self.vars.clone();
        new_vars.append(&mut rhs.vars);
        MultilinearMonomial::new(self.coefficient * rhs.coefficient, new_vars)
    }

    // Converts a multilinear monomial to a multilinear polynomial
    pub fn from(self) -> MultilinearPolynomial<F> {
        MultilinearPolynomial { terms: vec![self] }
    }
}

// Multilinear polynomial implementation
impl<F: PrimeField> MultilinearPolynomial<F> {
    // Creates new multilinear polynomial
    pub fn new(terms: Vec<MultilinearMonomial<F>>) -> Self {
        Self { terms }
    }

    // Removes empty terms from a multilinear polynomial
    pub fn truncate(mut self) -> MultilinearPolynomial<F> {
        match self.terms.pop() {
            Option::Some(val) => {
                if val.coefficient == F::zero() {
                    self.truncate()
                } else {
                    self.terms.push(val);
                    self
                }
            }
            Option::None => self,
        }
    }

    //Adds extra variables to a multilinear polynomial
    pub fn pad_vars(&self, total_vars: usize) -> MultilinearPolynomial<F> {
        let mut res = MultilinearPolynomial::new(vec![]);
        let diff = total_vars - self.terms[0].vars.len();
        let diff_vec = vec![false; diff];
        if diff > 0 {
            for mut term in self.terms.clone() {
                term.vars.extend(diff_vec.clone());
                res.terms.push(term);
            }
        };
        return res;
    }

    // Add like terms in a multilinear polynomial
    fn simplify(&self) -> MultilinearPolynomial<F> {
        let mut terms_map = BTreeMap::<usize, (F, Vec<bool>)>::new();

        let mut res = MultilinearPolynomial::new(vec![]);

        for i in 0..self.terms.len() {
            let selector = selector_to_index(&self.terms[i].vars);

            match terms_map.get(&selector) {
                Option::Some((coeffs, var)) => {
                    terms_map.insert(selector, (self.terms[i].coefficient + coeffs, var.clone()));
                }
                Option::None => {
                    terms_map.insert(
                        selector,
                        (self.terms[i].coefficient, self.terms[i].vars.clone()),
                    );
                }
            }
        }
        for (coeffs, var) in terms_map.values() {
            res.terms.push(MultilinearMonomial {
                coefficient: coeffs.clone(),
                vars: var.clone(),
            });
        }
        res
    }

    // Multiply a multilinear polynomial by a scalar value
    pub fn scalar_mul(&mut self, scalar: F) {
        for i in 0..self.terms.len() {
            self.terms[i].coefficient *= scalar;
        }
    }

    // Returns true of polynomial is a zero polynomial
    pub fn is_zero(&self) -> bool {
        self.terms.len() == 0
    }

    // Interpolate a polynomial using the boolean hypercube
    pub fn interpolate(y: Vec<F>) -> Self {
        let mut res = MultilinearPolynomial::new(vec![]);
        // let max_bit_count = (y.len() as f64).log2().ceil() as usize;
        let max_bit_count = format!("{:b}", y.len() - 1).len();
        for i in 0..y.len() {
            let mut y_multi_lin_poly =
                MultilinearPolynomial::new(vec![MultilinearMonomial::new(y[i], vec![])]);
            let boolean_hypercube = get_binary_string(i, max_bit_count);
            for j in 0..boolean_hypercube.len() {
                let i_char = boolean_hypercube.chars().nth(j).unwrap();
                if i_char == '0' {
                    let i_rep = check_bit(0);
                    y_multi_lin_poly = y_multi_lin_poly * i_rep;
                };
                if i_char == '1' {
                    let i_rep = check_bit(1);
                    y_multi_lin_poly = y_multi_lin_poly * i_rep;
                }
            }
            res = res.add(y_multi_lin_poly);
        }
        res
    }
}

// Implement native addition for Multilinear Polynomial
impl<F: PrimeField> Add for MultilinearPolynomial<F> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let mut res = self.clone();
        res.terms.extend(rhs.terms);
        res.simplify()
    }
}

// Implement native multiplication for multilinear polynomial
impl<F: PrimeField> Mul for MultilinearPolynomial<F> {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self::Output {
        let mut res = MultilinearPolynomial::new(vec![]);
        for i in 0..self.terms.len() {
            for j in 0..rhs.terms.len() {
                res.terms
                    .push(self.terms[i].multiply(&mut rhs.terms[j].clone()));
            }
        }
        res.simplify()
    }
}

pub trait MultilinearPolynomialTrait<F: PrimeField> {
    fn partial_eval(&self, x: &Vec<(usize, F)>) -> Self;
    fn evaluate(&self, x: &Vec<(usize, F)>) -> F;
    fn number_of_vars(&self) -> usize;
    fn to_bytes(&self) -> Vec<u8>;
    fn relabel(&self) -> Self;
    fn additive_identity() -> Self;
    fn sum_over_the_boolean_hypercube(&self) -> F;
}

impl<F: PrimeField> MultilinearPolynomialTrait<F> for MultilinearPolynomial<F> {
    // Partially evaluate a multilinear polynomial
    // tuple comprise of variable index which is equivalent to the variable
    // and Field element which is the point to evaluate at
    fn partial_eval(&self, x: &Vec<(usize, F)>) -> Self {
        let mut res = self.clone();
        for i in 0..res.terms.len() {
            for j in 0..x.len() {
                let (var, val) = x[j];
                assert!(var <= res.number_of_vars(), "Variable not found");
                if res.terms[i].vars[var] {
                    res.terms[i].coefficient *= val;
                    res.terms[i].vars[var] = false;
                };
            }
        }
        res.simplify()
    }

    // Relabels the variables to account for variables that have been evaluated
    fn relabel(&self) -> Self {
        let mut res = MultilinearPolynomial::<F>::new(vec![]);

        let mut label_checker = vec![false; self.number_of_vars()];

        for i in 0..self.terms.len() {
            label_checker = self.terms[i]
                .vars
                .iter()
                .zip(label_checker.iter())
                .map(|(a, b)| a | b)
                .collect();
        }

        for i in 0..self.terms.len() {
            let mut new_vars = vec![];
            for (a, b) in self.terms[i].vars.iter().zip(label_checker.iter()) {
                if *b {
                    new_vars.push(a.clone());
                }
            }
            let term = MultilinearMonomial::<F>::new(self.terms[i].coefficient, new_vars);
            res.terms.push(term);
        }

        res
    }

    // Fully evaluates a multilinear polynomial
    fn evaluate(&self, x: &Vec<(usize, F)>) -> F {
        let mut res = self.clone();
        if res.is_zero() {
            return F::zero();
        }

        // dbg!(&self.number_of_vars());
        // dbg!(&x.len());

        assert!(
            x.len() >= self.number_of_vars(),
            "Must evaluate at all points"
        );


        for i in 0..res.terms.len() {
            for j in 0..self.number_of_vars() {
                let (var, val) = x[j];
                assert!(var <= self.terms[0].vars.len(), "Variable not found");
                if res.terms[i].vars[var] {
                    res.terms[i].coefficient *= val;
                    res.terms[i].vars[var] = false;
                };
            }
        }
        assert!(
            res.simplify().terms.len() == 1,
            "All variables should be evaluated"
        );
        res.simplify().terms[0].coefficient
    }

    fn number_of_vars(&self) -> usize {
        if self.terms.len() == 0 {
            return 0;
        }
        self.terms[0].vars.len()
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::new();
        for i in 0..self.terms.len() {
            res.append(&mut self.terms[i].coefficient.into_bigint().to_bytes_be());
            res.append(
                &mut self.terms[i]
                    .vars
                    .iter()
                    .map(|a| *a as u8)
                    .collect::<Vec<u8>>(),
            );
        }
        res
    }

    fn additive_identity() -> Self {
        MultilinearPolynomial::new(vec![])
    }

    // Evaluates the sum over the boolean hypercube and returns the sum
    fn sum_over_the_boolean_hypercube(&self) -> F {
        let mut res = F::zero();
        if self.terms.len() == 0 {
            return F::zero();
        };
        let vars_len = self.terms[0].vars.len();
        for i in 0..2_usize.pow(vars_len as u32) {
            let boolean_vec: Vec<F> = get_binary_string(i, vars_len)
                .chars()
                .into_iter()
                .map(|var| if var == '0' { F::zero() } else { F::one() })
                .collect();
            let eval_domain = boolean_vec.into_iter().enumerate().collect();
            res += self.evaluate(&eval_domain);
        }
        res
    }
}

// TODO
// From converts a univariate polynomial to a multilinear polynomial
// impl <F: Field> TryFrom<MultilinearPolynomial<F>> for Polynomial<F> {
//     type Error = &'static str;

//     fn try_from(value: MultilinearPolynomial<F>) -> Result<Self, Self::Error> {
//         let mut res = Polynomial::<F>::new(vec![]);

//         if value.terms.len() == 0 {
//             res = Polynomial { coefficients: vec![] };
//         } else if value.terms[0].vars.len() > 1 {
//             return Err("Not a univariate poly, try relabelling");
//         } else {
//             if value.terms[0].vars[0] == false {
//                 res = Polynomial::<F>::new(vec![value.terms[0].coefficient, value.terms[1].coefficient]);
//             } else {
//                 res = Polynomial::<F>::new(vec![value.terms[1].coefficient, value.terms[0].coefficient]);
//             }
//         }

//         Ok(res)
//     }
// }

////////////////////////////////////
///  TESTS
/// ////////////////////////////////
#[cfg(test)]
mod tests {
    use ark_ff::{Fp64, MontBackend, MontConfig};

    use crate::polynomials::multilinear_poly::MultilinearPolynomialTrait;

    use super::{MultilinearMonomial, MultilinearPolynomial};

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    pub struct FqConfig;
    pub type Fq = Fp64<MontBackend<FqConfig, 1>>;

    #[test]
    fn test_add_multilinear_same() {
        let term1 = MultilinearMonomial::new(Fq::from(5), vec![true, false, true]);

        let multilin_poly = MultilinearPolynomial::new(vec![term1]);

        let res =
            ((multilin_poly.clone() + multilin_poly.clone()).truncate() + multilin_poly).truncate();

        assert_eq!(
            res,
            MultilinearPolynomial::new(vec![MultilinearMonomial::new(
                Fq::from(15),
                vec![true, false, true]
            )]),
            "Incorrect result"
        );
    }

    #[test]
    fn test_add_multilinear_diff() {
        let term1 = MultilinearMonomial::new(Fq::from(5), vec![true, false, true]); // 5ac
        let term2 = MultilinearMonomial::new(Fq::from(5), vec![true, true, true]); // 5abc

        let multilin_poly1 = MultilinearPolynomial::new(vec![term1.clone(), term2.clone()]); // 5ac + 5abc
        let multilin_poly2 = MultilinearPolynomial::new(vec![term2, term1]); //5abc + 5ac

        let res = (multilin_poly1.clone() + multilin_poly2.clone()).truncate();

        assert_eq!(
            res,
            MultilinearPolynomial::new(vec![
                MultilinearMonomial::new(Fq::from(10), vec![true, false, true]),
                MultilinearMonomial::new(Fq::from(10), vec![true, true, true])
            ]),
            "Incorrect result"
        ); // Result should equal -> 10ac + 10abc
    }

    #[test]
    fn test_partial_eval() {
        let term1 = MultilinearMonomial::new(Fq::from(3), vec![true, true, false]); // 3ab
        let term2 = MultilinearMonomial::new(Fq::from(8), vec![false, true, true]); // 8bc
        let multi_lin_poly = MultilinearPolynomial::new(vec![term1, term2]); // 3ab + 8bc

        let res = MultilinearPolynomialTrait::partial_eval(&multi_lin_poly, &vec![(1, Fq::from(3))]); // evaluating at b = 3
        assert_eq!(
            res,
            MultilinearPolynomial::new(vec![
                MultilinearMonomial::new(Fq::from(9), vec![true, false, false]),
                MultilinearMonomial::new(Fq::from(24), vec![false, false, true])
            ])
        ); // Res = 9a + 24c
    }

    // #[test]
    fn test_partial_eval_many() -> MultilinearPolynomial<Fq> {
        let term1 = MultilinearMonomial::new(Fq::from(3), vec![true, true, false]); // 3ab
        let term2 = MultilinearMonomial::new(Fq::from(8), vec![false, true, true]); // 8bc
        let multi_lin_poly = MultilinearPolynomial::new(vec![term1, term2]); // 3ab + 8bc

        let res = multi_lin_poly.partial_eval(&vec![(1, Fq::from(3)), (2, Fq::from(2))]); // evaluating at b = 3 and c = 2
        assert_eq!(
            res,
            MultilinearPolynomial::new(vec![
                MultilinearMonomial::new(Fq::from(48), vec![false, false, false]),
                MultilinearMonomial::new(Fq::from(9), vec![true, false, false]),
            ])
        ); // Res = 9a + 48

        res
    }

    #[test]
    fn test_evaluate() {
        let term1 = MultilinearMonomial::new(Fq::from(3), vec![true, true, false]); // 3ab
        let term2 = MultilinearMonomial::new(Fq::from(8), vec![false, true, true]); // 8bc
        let multi_lin_poly = MultilinearPolynomial::new(vec![term1, term2]); // 3ab + 8bc

        let res =
            multi_lin_poly.evaluate(&vec![(0, Fq::from(2)), (1, Fq::from(3)), (2, Fq::from(2))]); // evaluating at a = 2, b = 3 and c = 2
        assert_eq!(res, Fq::from(66)); // Res = 18 + 48
    }

    #[test]
    fn test_polynomial_scalar_multiplication() {
        let term1 = MultilinearMonomial::new(Fq::from(3), vec![true, true, false]); // 3ab
        let term2 = MultilinearMonomial::new(Fq::from(8), vec![false, true, true]); // 8bc
        let mut multi_lin_poly = MultilinearPolynomial::new(vec![term1, term2]); // 3ab + 8bc

        multi_lin_poly.scalar_mul(Fq::from(6));

        assert!(
            multi_lin_poly
                == MultilinearPolynomial::new(vec![
                    MultilinearMonomial::new(Fq::from(18), vec![true, true, false]),
                    MultilinearMonomial::new(Fq::from(48), vec![false, true, true])
                ])
        );
    }

    #[test]
    fn test_multilinear_monomial_mul() {
        let mut term1 = MultilinearMonomial::new(Fq::from(3), vec![true, true, false]); // 3ab
        let mut term2 = MultilinearMonomial::new(Fq::from(8), vec![false, true, true]); // 8bc
        let res = term1.multiply(&mut term2);
        dbg!(res.clone());
        assert!(
            res == MultilinearMonomial::new(
                Fq::from(24),
                vec![true, true, false, false, true, true]
            )
        );
    }

    #[test]
    fn test_multilinear_polynomial_mul() {
        let term1 = MultilinearMonomial::new(Fq::from(3), vec![true, true, false]); // 3ab
        let term2 = MultilinearMonomial::new(Fq::from(8), vec![false, true, true]); // 8bc
        let multi_lin_poly = MultilinearPolynomial::new(vec![term1, term2]); // 3ab + 8bc

        let res = multi_lin_poly.clone() * multi_lin_poly;
        // 3ab(3de + 8ef) + 8bc(3de + 8ef)
        // 9abde + 24abef + 24bcde + 64bcef
        dbg!(res.clone());
        assert!(
            res == MultilinearPolynomial::new(vec![
                MultilinearMonomial::new(Fq::from(9), vec![true, true, false, true, true, false]),
                MultilinearMonomial::new(Fq::from(24), vec![false, true, true, true, true, false]),
                MultilinearMonomial::new(Fq::from(24), vec![true, true, false, false, true, true]),
                MultilinearMonomial::new(Fq::from(64), vec![false, true, true, false, true, true]),
            ])
        );
    }

    // #[test]
    // fn test_multilinear_polynomial_mul_3vars() {
    //     let term1 = MultilinearMonomial::new(Fq::from(3), vec![true, true, false]); // 3ab
    //     let term2 = MultilinearMonomial::new(Fq::from(8), vec![false, true, true]); // 8bc
    //     let term3 = MultilinearMonomial::new(Fq::from(5), vec![true, false, true]); // 5ac
    //     let mut multi_lin_poly1 = MultilinearPolynomial::new(vec![term1, term2, term3]); // 3ab + 8bc + 5ac

    //     let term4 = MultilinearMonomial::new(Fq::from(3), vec![true, true, false]); // 3ab
    //     let term5 = MultilinearMonomial::new(Fq::from(8), vec![false, true, true]); // 8bc
    //     let term6 = MultilinearMonomial::new(Fq::from(5), vec![true, false, true]); // 5ac
    //     let mut multi_lin_poly2 = MultilinearPolynomial::new(vec![term4, term5, term6]); // 3ab + 8bc + 5ac

    //     let res = multi_lin_poly1.multiply(&mut multi_lin_poly2);
    //     // 3ab(3de + 8ef) + 8bc(3de + 8ef)
    //     // 9abde + 24abef + 24bcde + 64bcef
    //     dbg!(res.clone());
    //     assert!(
    //         res == MultilinearPolynomial::new(vec![
    //             MultilinearMonomial::new(Fq::from(9), vec![true, true, false, true, true, false]),
    //             MultilinearMonomial::new(Fq::from(24), vec![false, true, true, true, true, false]),
    //             MultilinearMonomial::new(Fq::from(24), vec![true, true, false, false, true, true]),
    //             MultilinearMonomial::new(Fq::from(64), vec![false, true, true, false, true, true]),
    //         ])
    //     );
    // }

    #[test]
    fn test_constant_poly() {
        let constant_poly =
            MultilinearPolynomial::new(vec![MultilinearMonomial::new(Fq::from(3), vec![])]);

        let term1 = MultilinearMonomial::new(Fq::from(3), vec![true, true, false]); // 3ab
        let term2 = MultilinearMonomial::new(Fq::from(8), vec![false, true, true]); // 8bc
        let multi_lin_poly = MultilinearPolynomial::new(vec![term1, term2]); // 3ab + 8bc

        let res = constant_poly * multi_lin_poly;

        assert!(
            res == MultilinearPolynomial::new(vec![
                MultilinearMonomial::new(Fq::from(9), vec![true, true, false]),
                MultilinearMonomial::new(Fq::from(24), vec![false, true, true])
            ])
        )
    }

    #[test]
    fn test_multilinear_interpolate() {
        let res_poly = MultilinearPolynomial::<Fq>::interpolate(vec![
            Fq::from(2),
            Fq::from(4),
            Fq::from(6),
            Fq::from(8),
            Fq::from(10),
        ]);
        assert!(&res_poly.terms[0].vars.len() == &3);
        assert!(
            res_poly
                .clone()
                .evaluate(&vec![(0, Fq::from(0)), (1, Fq::from(0)), (2, Fq::from(0))])
                == Fq::from(2)
        );
        assert!(
            res_poly
                .clone()
                .evaluate(&vec![(0, Fq::from(0)), (1, Fq::from(0)), (2, Fq::from(1))])
                == Fq::from(4)
        );
        assert!(
            res_poly
                .clone()
                .evaluate(&vec![(0, Fq::from(0)), (1, Fq::from(1)), (2, Fq::from(0))])
                == Fq::from(6)
        );
        assert!(
            res_poly
                .clone()
                .evaluate(&vec![(0, Fq::from(0)), (1, Fq::from(1)), (2, Fq::from(1))])
                == Fq::from(8)
        );
        assert!(
            res_poly
                .clone()
                .evaluate(&vec![(0, Fq::from(1)), (1, Fq::from(0)), (2, Fq::from(0))])
                == Fq::from(10)
        );
    }

    #[test]
    fn test_relabel() {
        let poly = test_partial_eval_many();
        dbg!(&poly);
        let new_poly = poly.relabel();
        dbg!(&new_poly);
        assert!(new_poly.terms[0].vars.len() == 1);
        assert!(new_poly.terms[0].vars == vec![false]);
        assert!(new_poly.terms[1].vars == vec![true]);
    }

    #[test]
    fn test_pad_vars() {
        let new_poly = MultilinearPolynomial::new(vec![
            MultilinearMonomial::new(Fq::from(2), vec![true, true]),
            MultilinearMonomial::new(Fq::from(2), vec![true, false]),
            MultilinearMonomial::new(Fq::from(2), vec![false, true]),
            MultilinearMonomial::new(Fq::from(2), vec![false, false]),
        ]);

        let res = new_poly.pad_vars(3);

        assert!(
            res == MultilinearPolynomial::new(vec![
                MultilinearMonomial::new(Fq::from(2), vec![true, true, false]),
                MultilinearMonomial::new(Fq::from(2), vec![true, false, false]),
                MultilinearMonomial::new(Fq::from(2), vec![false, true, false]),
                MultilinearMonomial::new(Fq::from(2), vec![false, false, false]),
            ]),
            "Padding failed"
        );
    }

    #[test]
    fn test_sum_over_boolean_hypercube() {
        let poly = MultilinearPolynomial::new(vec![
            MultilinearMonomial::new(Fq::from(2), vec![true, true, false]),
            MultilinearMonomial::new(Fq::from(3), vec![false, true, true]),
        ]);
        let res = poly.sum_over_the_boolean_hypercube();
        dbg!(&res);
        assert!(res == Fq::from(10))
    }
}
