import type { AppProps } from 'next/app';
import { LaserEyesProvider } from '@omnisat/lasereyes';
import '../styles/globals.css';

export default function App({ Component, pageProps }: AppProps) {
  return (
    <LaserEyesProvider config={{ network: 'mainnet' }}>
      <Component {...pageProps} />
    </LaserEyesProvider>
  );
} 