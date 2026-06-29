import React from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import { MantineProvider } from '@mantine/core';
import { DatesProvider } from '@mantine/dates';
import dayjs from 'dayjs';
import 'dayjs/locale/ja';
import '@mantine/core/styles.css';
import '@mantine/dates/styles.css';
import App from './App';
import { appTheme } from './theme/theme';

dayjs.locale('ja');

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <MantineProvider theme={appTheme} defaultColorScheme="light" forceColorScheme="light">
      <DatesProvider settings={{ locale: 'ja', firstDayOfWeek: 0 }}>
        <BrowserRouter>
          <App />
        </BrowserRouter>
      </DatesProvider>
    </MantineProvider>
  </React.StrictMode>,
);
