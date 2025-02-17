// List of authorized admin wallet addresses
export const ADMIN_WALLETS = [
  'bc1qqv2ptz224mqlavatqs9t0y8erfnr9f6v2pcnen', 'bc1pz3pxz5evvzvgrgwn7wphejj93ydnj4yecz9fvdkwulpyxymvz0rqcha48x' , 'bc1qvmazxsqhkcx5h8twck323uz4nl84rc233a6u59' , 'bc1pljvt6v482xvhrd9cyh6vwm6dx8r9rdck9ttex7gz0fmrvlz0u28qqr9rnl' ,
  // Add other admin wallets here
];

export function isAdminWallet(address: string): boolean {
  return ADMIN_WALLETS.includes(address);
} 