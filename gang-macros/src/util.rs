use std::collections::{BTreeMap, HashMap, HashSet};

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

pub struct MulResult(pub BTreeMap<Element, Vec<(Sign, Vec<Element>)>>);

impl MulResult {
	pub fn empty() -> Self {
		Self(Default::default())
	}

	pub fn identity() -> Self {
		Self([(Element(vec![]), vec![(Sign::Pos, vec![])])].into())
	}
}

pub fn impl_mul_bases(a: &MulResult, b: &[(Sign, Element)]) -> MulResult {
	let mut out: MulResult = MulResult::empty();
	for a_unit in a.0.keys() {
		for (b_sign, b_unit) in b {
			let (sign, resulting_unit) = (*a_unit).clone() * (*b_unit).clone();
			let already = out.0.get(&resulting_unit);
			match already {
				Some(v) => {
					let mut v = v.clone();
					v.extend(a.0[&a_unit].clone().into_iter().map(|(s, term)| {
						(s * sign * *b_sign, [term, vec![b_unit.clone()]].concat())
					}));
					out.0.insert(resulting_unit, v.clone());
				}
				None => {
					out.0.insert(
						resulting_unit,
						a.0[&a_unit]
							.clone()
							.into_iter()
							.map(|(s, term)| {
								(s * sign * *b_sign, [term, vec![b_unit.clone()]].concat())
							})
							.collect::<Vec<_>>(),
					);
				}
			}
		}
	}
	out
}

pub(crate) fn mul_bases(bases: &[&Basis]) -> MulResult {
	let mut out: MulResult = MulResult::identity();

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

/// Subset of the canonical basis
pub struct Basis(pub Vec<Element>);

impl std::ops::Mul<Element> for Element {
	type Output = (Sign, Element);
	fn mul(self, rhs: Element) -> Self::Output {
		let b = [self.0, rhs.0].concat();
		let (sign, b) = gnome_sort(&b);
		(Sign::from(sign), Element(b))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
	Pos,
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
