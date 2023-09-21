use std::collections::{HashMap, HashSet};

use proc_macro2::Ident;
use quote::format_ident;

use crate::{BasisOld, Element, MvKind};

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

pub(crate) fn mul_bases(
	x0: &BasisOld,
	x1: &BasisOld,
) -> (HashMap<Vec<u32>, Vec<(bool, Vec<u32>, Vec<u32>)>>, MvKind) {
	let mut pairs = Vec::new();
	for i in 0..x0.len() {
		for j in 0..x1.len() {
			pairs.push([&x0[i], &x1[j]]);
		}
	}
	let mut map: HashMap<Vec<u32>, Vec<(bool, Vec<u32>, Vec<u32>)>> = HashMap::new();
	let mut set = HashSet::new();
	for i in 0..pairs.len() {
		let mut b = pairs[i][0].clone();
		b.append(&mut pairs[i][1].clone());

		let (sign, b) = gnome_sort(&b);
		set.insert(b.clone());

		let already = map.get(&b);
		match already {
			Some(v) => {
				let mut v = v.clone();
				v.push((sign == -1, pairs[i][0].clone(), pairs[i][1].clone()));
				map.insert(b, v.clone());
			}
			None => {
				map.insert(
					b,
					vec![(sign == -1, pairs[i][0].clone(), pairs[i][1].clone())],
				);
			}
		}
	}
	(map.clone(), infer(set))
}

pub(crate) fn mul_bases2(bases: &[&Basis]) -> HashMap<Unit, Vec<(Sign, Vec<Unit>)>> {
	type Term = HashMap<Unit, Vec<(Sign, Vec<Unit>)>>;

	fn impl_mul_bases(a: &Term, b: &Basis) -> Term {
		let mut out: Term = Default::default();
		for a_unit in a.keys() {
			for b_unit in &b.0 {
				let (sign, resulting_unit) = (*a_unit).clone() * (*b_unit).clone();

				println!("{:?}*{:?}={:?}{:?}", a_unit, b_unit, sign, resulting_unit);

				let already = out.get(&resulting_unit);
				match already {
					Some(v) => {
						let mut v = v.clone();
						v.extend(
							a[&a_unit]
								.clone()
								.into_iter()
								.map(|(s, term)| (s * sign, [term, vec![b_unit.clone()]].concat())),
						);
						out.insert(resulting_unit, v.clone());
					}
					None => {
						out.insert(
							resulting_unit,
							a[&a_unit]
								.clone()
								.into_iter()
								.map(|(s, term)| (s * sign, [term, vec![b_unit.clone()]].concat()))
								.collect::<Vec<_>>(),
						);
					}
				}
			}
		}
		out
	}

	let mut out: Term = [(Unit(vec![]), vec![(Sign::Pos, vec![])])].into();

	for b in bases.iter().map(|base| base) {
		println!("{:?}", out);
		out = impl_mul_bases(&out, &b);
	}
	println!("{:?}", out);

	out
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Unit(pub Vec<u32>);
pub struct Basis(pub Vec<Unit>);

impl std::ops::Mul<Unit> for Unit {
	type Output = (Sign, Unit);
	fn mul(self, rhs: Unit) -> Self::Output {
		let b = [self.0, rhs.0].concat();
		let (sign, b) = gnome_sort(&b);
		(Sign::from(sign), Unit(b))
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

pub(crate) fn element_name(base: &[u32]) -> Ident {
	format_ident!(
		"e{}",
		base.into_iter().map(|i| i.to_string()).collect::<String>()
	)
}

pub(crate) fn element_name_upper(base: &[u32]) -> Ident {
	format_ident!(
		"E{}",
		base.into_iter().map(|i| i.to_string()).collect::<String>()
	)
}

pub(crate) fn basis_names(basis: &[Element]) -> Vec<Ident> {
	basis.iter().map(|e| element_name(e)).collect()
}

pub(crate) fn infer<B: IntoIterator<Item = Element>>(basis: B) -> MvKind {
	let grades: HashSet<usize> = basis.into_iter().map(|e| e.len()).collect();
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
