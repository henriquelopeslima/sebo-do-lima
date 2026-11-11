const DEFAULT_PORT: u16 = 3000;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
}

impl Config {
    /// Lê `DATABASE_URL` (obrigatória) e `PORT` (opcional) do ambiente.
    ///
    /// # Errors
    ///
    /// Retorna a mensagem do problema se `DATABASE_URL` não está definida ou `PORT` não é uma porta válida.
    pub fn from_env() -> Result<Self, String> {
        Self::from_vars(
            std::env::var("DATABASE_URL").ok(),
            std::env::var("PORT").ok(),
        )
    }

    /// Monta a configuração a partir dos valores brutos das variáveis.
    ///
    /// # Errors
    ///
    /// Mesmas condições de [`Config::from_env`].
    pub fn from_vars(database_url: Option<String>, port: Option<String>) -> Result<Self, String> {
        let database_url = database_url.ok_or("DATABASE_URL não definida")?;
        let port = match port {
            Some(port) => port.parse().map_err(|_| format!("PORT inválida: {port}"))?,
            None => DEFAULT_PORT,
        };
        Ok(Self { database_url, port })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_variaveis_e_aplica_padrao() {
        let config = Config::from_vars(Some("postgres://x".into()), None).unwrap();
        assert_eq!(config.database_url, "postgres://x");
        assert_eq!(config.port, 3000);
        let custom = Config::from_vars(Some("postgres://x".into()), Some("8081".into())).unwrap();
        assert_eq!(custom.port, 8081);
    }

    #[test]
    fn rejeita_configuracao_invalida() {
        assert!(
            Config::from_vars(None, None)
                .unwrap_err()
                .contains("DATABASE_URL")
        );
        assert!(
            Config::from_vars(Some("postgres://x".into()), Some("abc".into()))
                .unwrap_err()
                .contains("PORT")
        );
    }
}
