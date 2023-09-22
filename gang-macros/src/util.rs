use std::{
	collections::{BTreeMap, HashSet},
	fmt::Display,
};

use proc_macro2::Ident;
use quote::format_ident;

use crate::MvKind;

pub(crate) fn gnome_sort(input: &[u32]) -> (i32, Vec<u32>) {
	let mut v = input.to_vec();
	let mut i = 0;
	let mut sign = 1;
	while i < v.len() {
		if i == 0 || v[i - 1] <= v[i] {
			i += 1;
		} else {
			v.swap(i - 1, i);
			sign = -sign;
			i -= 1;
		}
	}
	for i in (1..v.len()).rev() {
		if i >= v.len() {
			continue;
		}
		if v[i - 1] == v[i] {
			v.drain(i - 1..i + 1);
		}
	}
	(sign, v)
}

#[derive(Debug, Clone)]
pub struct LinearCombination(pub Vec<(Sign, Vec<Element>)>);

impl Display for LinearCombination {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let out = self
			.0
			.iter()
			.map(|(s, els)| {
				let sign_str = match s {
					Sign::Pos => "+",
					Sign::Neg => "-",
				};
				let els_str = (els.len() > 0)
					.then_some(
						els.iter()
							.map(|el| format!("{}", el))
							.collect::<Vec<_>>()
							.join(" "),
					)
					.unwrap_or(String::from("1"));
				format!("{} {}", sign_str, els_str)
			})
			.collect::<Vec<_>>()
			.join(" ");
		write!(f, "{}", out)
	}
}

impl LinearCombination {
	pub fn one() -> Self {
		Self(vec![(Sign::Pos, vec![])])
	}
}

#[derive(Debug)]
pub struct LinearCombinations(pub BTreeMap<Element, LinearCombination>);

impl Display for LinearCombinations {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "LinearCombination {{\n")?;
		for (el, combination) in &self.0 {
			write!(f, "\t{}: {},\n", el, combination)?;
		}
		write!(f, "}}")
	}
}

impl LinearCombinations {
	pub fn zero() -> Self {
		Self(Default::default())
	}

	pub fn one() -> Self {
		Self([(Element(vec![]), LinearCombination::one())].into())
	}
}

pub fn impl_mul_bases(a: &LinearCombinations, b: &[(Sign, Element)]) -> LinearCombinations {
	let mut out: LinearCombinations = LinearCombinations::zero();
	for a_unit in a.0.keys() {
		for (b_sign, b_unit) in b {
			let (sign, resulting_unit) = (*a_unit).clone() * (*b_unit).clone();
			let already = out.0.get(&resulting_unit);
			match already {
				Some(v) => {
					let mut v = v.clone();
					v.0.extend(a.0[&a_unit].clone().0.into_iter().map(|(s, term)| {
						(s * sign * *b_sign, [term, vec![b_unit.clone()]].concat())
					}));
					out.0.insert(resulting_unit, v.clone());
				}
				None => {
					out.0.insert(
						resulting_unit,
						LinearCombination(
							a.0[&a_unit]
								.clone()
								.0
								.into_iter()
								.map(|(s, term)| {
									(s * sign * *b_sign, [term, vec![b_unit.clone()]].concat())
								})
								.collect::<Vec<_>>(),
						),
					);
				}
			}
		}
	}
	out
}

pub(crate) fn mul_bases(bases: &[&Basis]) -> LinearCombinations {
	let mut out: LinearCombinations = LinearCombinations::one();

	for b in bases.iter().map(|base| base) {
		out = impl_mul_bases(
			&out,
			&b.0.iter()
				.map(|p| (Sign::Pos, p.clone()))
				.collect::<Vec<_>>(),
		);
	}
	out
}

/// A vector of the canonical basis
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Element(pub Vec<u32>);
impl Element {
	pub fn grade(&self) -> usize {
		self.0.len()
	}
}

impl Display for Element {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "e")?;
		for v in &self.0 {
			write!(f, "{}", v)?;
		}
		Ok(())
	}
}

/// Subset of the canonical basis
#[derive(Debug)]
pub struct Basis(pub Vec<Element>);

impl std::ops::Mul<Element> for Element {
	type Output = (Sign, Element);
	fn mul(self, rhs: Element) -> Self::Output {
		let b = [self.0, rhs.0].concat();
		let (sign, b) = gnome_sort(&b);
		(Sign::from(sign), Element(b))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
pub enum Sign {
	#[display(fmt = "+")]
	Pos,
	#[display(fmt = "-")]
	Neg,
}

impl std::ops::Mul<Sign> for Sign {
	type Output = Sign;

	fn mul(self, rhs: Sign) -> Self::Output {
		match self == rhs {
			true => Sign::Pos,
			false => Sign::Neg,
		}
	}
}

impl From<i32> for Sign {
	fn from(value: i32) -> Self {
		match value >= 0 {
			true => Sign::Pos,
			false => Sign::Neg,
		}
	}
}

pub(crate) fn element_name(base: &Element) -> Ident {
	format_ident!(
		"e{}",
		base.0.iter().map(|i| i.to_string()).collect::<String>()
	)
}

pub(crate) fn element_name_upper(base: &Element) -> Ident {
	format_ident!(
		"E{}",
		base.0.iter().map(|i| i.to_string()).collect::<String>()
	)
}

pub(crate) fn basis_names(basis: &Basis) -> Vec<Ident> {
	basis.0.iter().map(|e| element_name(e)).collect()
}

pub(crate) fn infer<B: IntoIterator<Item = Element>>(basis: B) -> MvKind {
	let grades: HashSet<usize> = basis.into_iter().map(|e| e.0.len()).collect();
	let odd_grades: HashSet<usize> = grades.iter().filter(|&x| x % 2 != 0).map(|&x| x).collect();
	if grades.len() > 1 && odd_grades.len() == 0 {
		MvKind::Rotor
	} else if grades.len() > 1 {
		MvKind::General
	} else if grades.len() == 1 {
		MvKind::KVector(grades.into_iter().next().unwrap())
	} else {
		panic!("multivector with no grade aren't supposed to happen");
	}
}
