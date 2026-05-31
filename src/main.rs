fn main() {
    println!("Hello, world!");

    println!("El resultado de la suma es: {}", plus(12, 12));
}

pub fn plus(a:i32, b:i32) -> i32 {
    a+b
}

// 1. Lógica central de tu Spooler Agent (Ejemplo)
pub fn validar_id_impresion(id: &str) -> bool {
    id.starts_with("SPOOL-")
}

// 2. Módulo de pruebas requerido para cumplir con tu suite mínima [cite: 46]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_valido() {
        assert!(validar_id_impresion("-12345"));
    }

    #[test]
    fn test_id_invalido() {
        assert!(!validar_id_impresion("PRINT-12345"));
    }
}

