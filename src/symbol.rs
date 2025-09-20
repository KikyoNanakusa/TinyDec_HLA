use goblin::elf::Elf;

pub struct Symbol {
	pub name: String,
	pub addr: u64,
}

impl Symbol {
	pub fn from_elf(elf: &Elf) -> Vec<Symbol> {
		let mut symbols = Vec::new();
		for sym in elf.syms.iter() {
			if let Some(name) = elf.strtab.get_at(sym.st_name) {
				if sym.is_function() && sym.st_value != 0 {
					symbols.push(Symbol {
						name: name.to_string(),
						addr: sym.st_value,
					});
				}
			}
		}
		symbols.sort_by_key(|s| s.addr);
		symbols
	}
}
