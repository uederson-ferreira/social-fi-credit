const { EnvironmentPlugin } = require('webpack');

module.exports = {
  webpack: {
    configure: {
      resolve: {
        fallback: {
          path: require.resolve('path-browserify'),
          crypto: require.resolve('crypto-browserify'),
          stream: require.resolve('stream-browserify'),
          buffer: require.resolve('buffer'),
        },
      },
    },
    plugins: [
      new EnvironmentPlugin({
        process: { env: {} },
      }),
    ],
  },
};