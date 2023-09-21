use std::collections::{HashMap, HashSet};

use proc_macro2::Ident;
use quote::format_ident;

use crate::{Basis, Element, MvKind};

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
	x0: &Basis,
	x1: &Basis,
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
