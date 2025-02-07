import type { AppProps } from 'next/app';
import '../styles/globals.css';
import { LaserEyesProvider } from '@omnisat/lasereyes';

export default function App({ Component, pageProps }: AppProps) {
  return (
    <LaserEyesProvider config={{ network: 'mainnet' }}>
      <Component {...pageProps} />
    </LaserEyesProvider>
  );
} 