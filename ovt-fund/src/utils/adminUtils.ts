// List of authorized admin wallet addresses
export const ADMIN_WALLETS = [
  'bc1qqv2ptz224mqlavatqs9t0y8erfnr9f6v2pcnen', 'bc1pz3pxz5evvzvgrgwn7wphejj93ydnj4yecz9fvdkwulpyxymvz0rqcha48x' ,
  // Add other admin wallets here
];

export function isAdminWallet(address: string): boolean {
  return ADMIN_WALLETS.includes(address);
} 