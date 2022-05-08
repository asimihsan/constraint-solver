const { merge } = require('webpack-merge');
const common = require('./webpack.common.js');

module.exports = merge(common, {
    mode: 'development',
    devtool: 'inline-source-map',
    cache: {
        type: 'filesystem',
        hashAlgorithm: 'sha256',
    },
    devServer: {
        static: {
            publicPath: './dist/',
            watch: true,
        },
        devMiddleware: {
            writeToDisk: true,
        },
        port: 8086,
    },
});
