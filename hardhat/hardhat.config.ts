/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  networks: {
    hardhat: {
      forking: {
        url: '', // e.g. 'https://eth-mainnet.g.alchemy.com/v2/your-key'
        chainId: 1,
      },
    },
  },
};
