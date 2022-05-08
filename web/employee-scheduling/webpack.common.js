const path = require('path');

const CopyPlugin = require("copy-webpack-plugin");
const CssMinimizerPlugin = require('css-minimizer-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

module.exports = {
    entry: {
        index: './src/index.ts',
    },
    resolve: {
        extensions: ['.ts', '.tsx', '.js', '.jsx', '.mjs', '.json', '.wasm'],
    },
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: '[name]/[name].[contenthash].js',
        publicPath: 'auto',
        clean: false,
    },

    // See: https://webpack.js.org/guides/caching/
    optimization: {
        moduleIds: 'deterministic',
        runtimeChunk: 'single',
        splitChunks: {
            cacheGroups: {
                vendor: {
                    test: /[\\/]node_modules[\\/]/,
                    name: 'vendors',
                    chunks: 'all',
                }
            }
        },
        minimize: true,
        minimizer: [
            `...`,
            new CssMinimizerPlugin(),
        ],
    },
    module: {
        rules: [
            {
                test: /\.(mjs|js|jsx|ts|tsx)$/,
                exclude: /node_modules/,
                use: {
                    loader: 'babel-loader',
                    options: {
                        presets: [
                            '@babel/preset-env',
                        ],
                        plugins: [
                            ["@babel/transform-runtime", {
                                "regenerator": true,
                            }],
                        ]
                    }
                }
            },
            {
                test: /\.css$/,
                use: [MiniCssExtractPlugin.loader, 'css-loader'],
            },
            {
                test: /\.(png|svg|jpg|jpeg|gif)$/i,
                type: 'asset/resource',
            },
            {
                test: /\.(woff|woff2|eot|ttf|otf)$/i,
                type: 'asset/resource',
            },
            {
                test: /\.wasm$/,
                type: 'webassembly/sync',
            }
        ]
    },
    experiments: {
        syncWebAssembly: true,
    },
    plugins: [
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, '..', 'employee-scheduling-wasm-bindgen'),
            watchDirectories: [
                path.resolve(__dirname, '..', '..', 'examples', 'employee-scheduling'),
                path.resolve(__dirname, '..', 'employee-scheduling-wasm-bindgen'),
            ],
            outDir: path.resolve(__dirname, 'src', 'pkg_employee_scheduling'),
            outName: "employee_scheduling",
            forceMode: "production",
        }),
        new MiniCssExtractPlugin({
            filename: '[name].[contenthash].css',
            chunkFilename: '[id].[contenthash].css',
        }),
        new CopyPlugin({
            patterns: [
                path.resolve(__dirname, 'src', 'favicon.ico'),
                path.resolve(__dirname, 'src', 'android-chrome-192x192.png'),
                path.resolve(__dirname, 'src', 'android-chrome-512x512.png'),
                path.resolve(__dirname, 'src', 'apple-touch-icon.png'),
                path.resolve(__dirname, 'src', 'favicon-16x16.png'),
                path.resolve(__dirname, 'src', 'favicon-32x32.png'),
            ],
        }),
        new HtmlWebpackPlugin({
            title: 'Employee Scheduling',
            template: path.resolve(__dirname, 'src', 'index.html'),
            filename: 'index.html',
            chunks: ['index'],
        })
    ]
};
