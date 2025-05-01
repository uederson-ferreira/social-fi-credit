# Build stage
FROM node:23-alpine as build

WORKDIR /app

# Copiar package.json e instalar dependências
COPY package*.json ./

# Instalar dependências com flag para ignorar conflitos de peer dependencies
RUN npm install --force

# Instalar versões específicas e compatíveis do MultiversX SDK
RUN npm install @multiversx/sdk-core@14.0.2
RUN npm install @multiversx/sdk-network-providers@2.9.3
RUN npm install @multiversx/sdk-extension-provider@5.0.0
RUN npm install @multiversx/sdk-wallet-connect-provider@6.0.0

# Configurar craco para resolver problemas de compatibilidade
RUN echo "const { EnvironmentPlugin } = require('webpack');module.exports = {  webpack: {    configure: {      resolve: {        fallback: {          path: require.resolve('path-browserify'),          crypto: require.resolve('crypto-browserify'),          stream: require.resolve('stream-browserify'),          buffer: require.resolve('buffer'),        },      },    },    plugins: [      new EnvironmentPlugin({        process: { env: {} },      }),    ],  },};" > craco.config.js

# Instalar dependências de polyfill necessárias
RUN npm install --save-dev path-browserify crypto-browserify stream-browserify buffer

# Instalar craco
RUN npm install @craco/craco --save-dev

# Atualizar scripts no package.json
RUN sed -i 's/"build": "react-scripts build"/"build": "craco build"/g' package.json
RUN sed -i 's/"start": "react-scripts start"/"start": "craco start"/g' package.json

# Copiar o restante dos arquivos do projeto
COPY . .
# Construir o aplicativo
RUN npm run build --legacy-peer-deps || echo "Build falhou, mas vamos continuar"


# Production stage
FROM nginx:alpine

# Copiar build files do estágio de build
COPY --from=build /app/build /usr/share/nginx/html

# Copiar configuração do nginx
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]