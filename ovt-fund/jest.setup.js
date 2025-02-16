// Learn more: https://github.com/testing-library/jest-dom
import '@testing-library/jest-dom'

// Mock next/router
jest.mock('next/router', () => ({
  useRouter() {
    return {
      route: '/',
      pathname: '',
      query: {},
      asPath: '',
      push: jest.fn(),
      replace: jest.fn(),
    }
  },
}))

// Mock Bitcoin price hook
jest.mock('./hooks/useBitcoinPrice', () => ({
  useBitcoinPrice: () => ({
    price: 40000,
    loading: false,
    error: null
  })
})) 