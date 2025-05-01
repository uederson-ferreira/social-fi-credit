# social-fi Credit

> Revolucionando DeFi com empréstimos sem colateral baseados em reputação social.

## 📝 Descrição

social-fi Credit é uma plataforma DeFi inovadora que utiliza reputação social como base para crédito, eliminando a necessidade de colateral tradicional. Através da integração com ElizaOS, o sistema monitora interações sociais em plataformas como Twitter/X, calculando um "Community Score" que determina a elegibilidade e os limites de empréstimo.

## ✨ Principais Recursos

- **Empréstimos Sem Colateral**: Acesso a crédito DeFi baseado em reputação social
- **Oráculo Social ElizaOS**: Monitoramento e análise de interações no Twitter via #ElizaOS
- **Community Score**: Sistema transparente de pontuação para qualificação de crédito
- **Pools de Liquidez**: Diferentes níveis de risco (AAA, BBB, CCC) para investidores
- **Tokenização de Dívidas**: NFTs representando empréstimos negociáveis
- **Sistema de Penalidades**: Mecanismos automatizados para incentivar pagamentos em dia

## 🚀 Instalação

```bash
# Clonar o repositório
git clone https://github.com/seu-usuario/social-fi-credit.git

# Entrar no diretório
cd social-fi-credit

# Instalar dependências
npm install

# Configurar variáveis de ambiente
cp .env.example .env
```

## 💻 Uso

### Configuração do ElizaOS

```bash
# Configurar credenciais do Twitter API
npm run config:eliza

# Iniciar monitoramento de hashtags
npm run start:monitor
```

### Deploy dos Contratos

```bash
# Compilar contratos
npx hardhat compile

# Deploy em testnet
npx hardhat run scripts/deploy.js --network mumbai
```

### Interface de Usuário

```bash
# Iniciar aplicação frontend
npm run start:app
```

## 🏗️ Arquitetura

### Smart Contracts

- `ReputationScore.sol`: Gerencia o score social dos usuários
- `LoanController.sol`: Controla aprovação e gestão de empréstimos
- `LiquidityPool.sol`: Administra pools de liquidez por nível de risco
- `DebtToken.sol`: Implementa NFTs representando dívidas

### ElizaOS Integration

- `TwitterMonitor`: Monitora interações com #ElizaOS
- `SentimentAnalyzer`: Analisa o sentimento das menções
- `ScoreCalculator`: Calcula o Community Score

## 🤝 Contribuição

Contribuições são bem-vindas! Por favor, leia nosso [guia de contribuição](CONTRIBUTING.md) para detalhes sobre nosso código de conduta e o processo de envio de pull requests.

## 📜 Licença

Este projeto está licenciado sob a [MIT License](LICENSE).

## 🔗 Links Úteis

- [Documentação Técnica](docs/technical.md)
- [Guia do Usuário](docs/user-guide.md)
- [FAQ](docs/faq.md)

---

Desenvolvido como parte do desafio Smartcontracts Challenger: DeFi Zero-Collateral Lending + ElizaOS.
