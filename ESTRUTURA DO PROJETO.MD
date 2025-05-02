
# Estrutura do Projeto Social-Fi Credit

## 1. Smart Contracts

### Pasta: `/smart-contracts/`

Contém todos os contratos inteligentes em Rust para a blockchain MultiversX.

#### 1.1. `/smart-contracts/reputation-score/`

**Função**: Gerencia os scores de reputação dos usuários baseado em dados sociais e financeiros.

* `Cargo.toml` - Arquivo de configuração e dependências Rust
* `/src/lib.rs` - Implementação principal do contrato ReputationScore
* `/wasm/` - Diretório para os arquivos compilados WebAssembly

#### 1.2. `/smart-contracts/loan-controller/`

**Função**: Gerencia o ciclo de vida dos empréstimos, incluindo solicitação, aprovação e pagamento.

* `Cargo.toml` - Arquivo de configuração e dependências Rust
* `/src/lib.rs` - Implementação principal do contrato LoanController
* `/wasm/` - Diretório para os arquivos compilados WebAssembly

#### 1.3. `/smart-contracts/liquidity-pool/`

**Função**: Gerencia o pool de liquidez onde investidores depositam fundos para financiar empréstimos.

* `Cargo.toml` - Arquivo de configuração e dependências Rust
* `/src/lib.rs` - Implementação principal do contrato LiquidityPool
* `/wasm/` - Diretório para os arquivos compilados WebAssembly

#### 1.4. `/smart-contracts/debt-token/`

**Função**: Implementa NFTs que representam dívidas, permitindo possível transferência ou negociação.

* `Cargo.toml` - Arquivo de configuração e dependências Rust
* `/src/lib.rs` - Implementação principal do contrato DebtToken
* `/wasm/` - Diretório para os arquivos compilados WebAssembly

#### 1.5. `/smart-contracts/tests/`

**Função**: Testes para os smart contracts.

* `reputation_score_test.rs` - Testes para o contrato ReputationScore
* `loan_controller_test.rs` - Testes para o contrato LoanController
* `liquidity_pool_test.rs` - Testes para o contrato LiquidityPool
* `debt_token_test.rs` - Testes para o contrato DebtToken

#### 1.6. `/smart-contracts/utils/`

**Função**: Utilitários compartilhados entre os contratos.

* `Cargo.toml` - Arquivo de configuração e dependências Rust
* `/src/lib.rs` - Implementação de funções auxiliares compartilhadas

## 2. Backend

### Pasta: `/backend/`

Implementação do servidor backend em Python, gerenciando a lógica de negócios e a comunicação com a blockchain.

#### 2.1. `/backend/eliza_os/`

**Função**: Componente responsável pela integração com dados sociais e cálculo de scores.

* `__init__.py` - Inicialização do módulo
* `twitter_monitor.py` - Monitoramento de hashtags e atividades no Twitter
* `sentiment_analyzer.py` - Análise de sentimento das interações sociais
* `score_calculator.py` - Algoritmo para cálculo do Community Score

#### 2.2. `/backend/api/`

**Função**: Implementação da API RESTful para comunicação com o frontend.

* `__init__.py` - Inicialização do módulo
* `main.py` - Ponto de entrada da API

##### 2.2.1. `/backend/api/routes/`

**Função**: Definição dos endpoints da API.

* `__init__.py` - Inicialização do módulo
* `users.py` - Endpoints para gerenciamento de usuários
* `loans.py` - Endpoints para gerenciamento de empréstimos
* `pools.py` - Endpoints para gerenciamento de pools de liquidez

##### 2.2.2. `/backend/api/models/`

**Função**: Definição dos modelos de dados.

* `__init__.py` - Inicialização do módulo
* `user.py` - Modelo para usuários
* `loan.py` - Modelo para empréstimos
* `pool.py` - Modelo para pools de liquidez

##### 2.2.3. `/backend/api/services/`

**Função**: Lógica de negócios.

* `__init__.py` - Inicialização do módulo
* `blockchain.py` - Serviço para comunicação com a blockchain
* `loan_service.py` - Serviço para operações de empréstimo
* `user_service.py` - Serviço para operações de usuário

#### 2.3. `/backend/oracle/`

**Função**: Serviço de oráculo para atualizar dados na blockchain.

* `__init__.py` - Inicialização do módulo
* `oracle_service.py` - Serviço principal do oráculo
* `score_updater.py` - Componente para atualizar scores na blockchain

#### 2.4. `/backend/config/`

**Função**: Configurações do backend.

* `__init__.py` - Inicialização do módulo
* `settings.py` - Configurações gerais
* `logging.py` - Configuração de logs

#### 2.5. `/backend/utils/`

**Função**: Utilitários para o backend.

* `__init__.py` - Inicialização do módulo
* `twitter_client.py` - Cliente para API do Twitter
* `multiversx_client.py` - Cliente para blockchain MultiversX

#### 2.6. `/backend/tests/`

**Função**: Testes para o backend.

* `__init__.py` - Inicialização do módulo
* `test_twitter_monitor.py` - Testes para monitoramento do Twitter
* `test_score_calculator.py` - Testes para cálculo de scores
* `test_oracle_service.py` - Testes para serviço de oráculo

#### 2.7. Arquivos na raiz do backend

* `requirements.txt` - Dependências Python
* `Dockerfile` - Configuração para containerização

## 3. Frontend

### Pasta: `/frontend/`

Interface de usuário em React com TypeScript.

#### 3.1. `/frontend/public/`

**Função**: Arquivos estáticos públicos.

* `index.html` - Ponto de entrada HTML
* `favicon.ico` - Ícone do site
* `/assets/images/` - Imagens estáticas

#### 3.2. `/frontend/src/`

**Função**: Código-fonte do frontend.

##### 3.2.1. `/frontend/src/components/`

**Função**: Componentes React reutilizáveis.

* `/common/` - Componentes comuns (Header, Footer, Button, Card)
* `/dashboard/` - Componentes do painel de controle
* `/loans/` - Componentes relacionados a empréstimos
* `/pools/` - Componentes relacionados a pools de liquidez

##### 3.2.2. `/frontend/src/pages/`

**Função**: Páginas principais da aplicação.

* `Home.tsx` - Página inicial
* `Dashboard.tsx` - Painel de controle do usuário
* `Loans.tsx` - Página de empréstimos
* `Pools.tsx` - Página de pools de liquidez
* `Profile.tsx` - Perfil do usuário
* `NFTMarketplace.tsx` - Mercado de NFTs de dívida

##### 3.2.3. `/frontend/src/hooks/`

**Função**: React Hooks customizados.

* `useWallet.tsx` - Hook para integração com carteira MultiversX
* `useLoans.tsx` - Hook para operações com empréstimos
* `useScore.tsx` - Hook para obtenção e atualização de scores

##### 3.2.4. `/frontend/src/services/`

**Função**: Serviços para comunicação com a API.

* `api.ts` - Cliente de API REST
* `wallet.ts` - Interface com carteira MultiversX
* `transactions.ts` - Gerenciamento de transações blockchain

##### 3.2.5. `/frontend/src/utils/`

**Função**: Funções auxiliares.

* `formatters.ts` - Formatadores de dados
* `validators.ts` - Validadores de entrada
* `constants.ts` - Constantes da aplicação

##### 3.2.6. `/frontend/src/contexts/`

**Função**: Contextos React para compartilhamento de estado.

* `WalletContext.tsx` - Contexto para carteira MultiversX
* `UserContext.tsx` - Contexto para dados do usuário

##### 3.2.7. `/frontend/src/types/`

**Função**: Definições de tipos TypeScript.

* `loan.ts` - Tipos para empréstimos
* `user.ts` - Tipos para usuários
* `pool.ts` - Tipos para pools de liquidez

##### 3.2.8. `/frontend/src/assets/`

**Função**: Recursos estáticos.

* `/styles/` - Arquivos CSS
* `/icons/` - Ícones da aplicação

##### 3.2.9. Arquivos na raiz de src

* `App.tsx` - Componente principal da aplicação
* `index.tsx` - Ponto de entrada JavaScript
* `routes.tsx` - Configuração de rotas

#### 3.3. Arquivos na raiz do frontend

* `package.json` - Dependências e scripts
* `tsconfig.json` - Configuração TypeScript
* `.eslintrc.js` - Configuração ESLint
* `tailwind.config.js` - Configuração Tailwind CSS
* `Dockerfile` - Configuração para containerização

## 4. Scripts e Documentação

### 4.1. `/scripts/`

**Função**: Scripts de utilidade para desenvolvimento e deploy.

* `deploy_contracts.sh` - Script para deploy dos contratos
* `setup_environment.sh` - Script para configuração do ambiente
* `run_tests.sh` - Script para execução de testes

### 4.2. `/docs/`

**Função**: Documentação do projeto.

* `technical.md` - Documentação técnica
* `user-guide.md` - Guia do usuário
* `api-docs.md` - Documentação da API
* `/contract-abi/` - ABIs dos contratos para integração
* `/diagrams/` - Diagramas de arquitetura

## 5. Arquivos na Raiz do Projeto

* `docker-compose.yml` - Configuração para orquestração de containers
* `.github/workflows/` - Configurações de CI/CD para GitHub Actions
* `.gitignore` - Arquivos a serem ignorados pelo Git
* `README.md` - Documentação geral do projeto

# Fluxo Principal de Funcionamento

1. Usuários se registram na plataforma conectando suas carteiras MultiversX
2. O componente ElizaOS monitora atividades sociais dos usuários (Twitter, etc.)
3. O analisador de sentimento processa essas atividades para avaliar o comportamento
4. O calculador de score gera um Community Score baseado nesses dados
5. O oráculo atualiza periodicamente os scores na blockchain
6. Usuários podem solicitar empréstimos no frontend
7. O contrato LoanController verifica a elegibilidade com base no score
8. Se aprovado, fundos são transferidos do LiquidityPool para o usuário
9. Um NFT é gerado pelo DebtToken representando a dívida
10. Usuários pagam empréstimos através do frontend
11. Pagamentos em dia melhoram o score do usuário.

**Testes Implementados
**1. Testes Unitários

LoanController: Testes para cada função individual, verificação de parâmetros e estados
ReputationScore: Testes para cálculo e gestão de pontuações
DebtToken: Testes para emissão, queima e transferência de tokens
LiquidityPool: Testes para depósitos, empréstimos e distribuição de juros

**2. Testes Fuzzy

LoanController: Testes com valores aleatórios para cálculos de taxas e empréstimos
ReputationScore: Testes com pontuações aleatórias e atualizações
DebtToken: Testes com transferências aleatórias e operações de tokens
LiquidityPool: Testes com depósitos e empréstimos aleatórios

**3. Testes de Segurança

LoanController: Testes contra ataques e manipulações
ReputationScore: Testes de acesso não autorizado e manipulação de pontuação
DebtToken: Testes contra reentrância e outros vetores de ataque
LiquidityPool: Testes de proteção contra diversos tipos de ataque

**4. Testes de Fluxo

LoanController: Cenários de fluxo completo de empréstimos
ReputationScore: Cenários de evolução de pontuação
DebtToken: Cenários de mercado secundário e recuperação
LiquidityPool: Cenários de emergência e ajuste dinâmico

**5. Testes Integrados

Testes de fluxo completo entre todos os componentes
Testes de múltiplos usuários e empréstimos
Testes de situações de crise e emergência
Testes de garantias e liquidações

Benefícios desta Abordagem Abrangente

Robustez: Identificação e correção de erros antes da implantação
Segurança: Proteção contra diversos vetores de ataque e vulnerabilidades
Confiabilidade: Garantia de que o sistema funciona conforme esperado em situações normais e adversas
Manutenibilidade: Facilidade para modificar e estender o sistema no futuro, com segurança de que as funcionalidades existentes continuam funcionando

Esta suíte completa de testes proporciona uma cobertura exaustiva para seu sistema de empréstimos, garantindo que ele seja seguro, confiável e robusto para uso em produção na blockchain MultiversX.
A abordagem em camadas (unitários → fuzzy → segurança → fluxo → integrados) permite identificar e corrigir problemas de forma estruturada, começando pelas funções individuais e progredindo para comportamentos mais complexos e interações entre componentes.
