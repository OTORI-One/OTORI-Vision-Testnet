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
jest.mock('@/src/hooks/useBitcoinPrice', () => ({
  useBitcoinPrice: () => ({
    price: 40000,
    loading: false,
    error: null
  })
}))

// Mock Headless UI Dialog components
jest.mock('@headlessui/react', () => {
  const Fragment = ({ children }) => children;

  const Dialog = function Dialog({ children, className, onClose, ...props }) {
    return (
      <div role="dialog" aria-modal="true" className={className} {...props}>
        {children}
      </div>
    );
  };

  Dialog.Panel = function Panel({ children, className, ...props }) {
    return (
      <div className={className} {...props}>
        {children}
      </div>
    );
  };

  Dialog.Title = function Title({ children, as: Component = 'h3', ...props }) {
    return <Component {...props}>{children}</Component>;
  };

  const Transition = {
    Root: function Root({ show, appear, children }) {
      return show ? children : null;
    },
    Child: function Child({ children, ...props }) {
      return children;
    }
  };

  Transition.Root = function TransitionRoot({ show, as: Component = Fragment, children }) {
    return show ? <Component>{children}</Component> : null;
  };

  return {
    Dialog,
    Transition,
    Fragment
  };
});

// Mock HeroIcons
jest.mock('@heroicons/react/24/outline', () => ({
  XMarkIcon: () => <span data-testid="close-icon">X</span>,
  ArrowTopRightOnSquareIcon: () => <span data-testid="external-link-icon">↗</span>,
  KeyIcon: () => <span data-testid="key-icon">🔑</span>,
  CheckCircleIcon: () => <span data-testid="check-circle-icon">✓</span>,
}))

// Mock window.matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: jest.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: jest.fn(), // deprecated
    removeListener: jest.fn(), // deprecated
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
    dispatchEvent: jest.fn(),
  })),
});

// Mock IntersectionObserver
const mockIntersectionObserver = jest.fn();
mockIntersectionObserver.mockImplementation(function (callback, options) {
  return {
    observe: () => null,
    unobserve: () => null,
    disconnect: () => null,
  };
});
window.IntersectionObserver = mockIntersectionObserver;

// Mock ResizeObserver
window.ResizeObserver = jest.fn().mockImplementation(() => ({
  observe: jest.fn(),
  unobserve: jest.fn(),
  disconnect: jest.fn(),
})); 