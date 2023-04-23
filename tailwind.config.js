/** @type {import('tailwindcss').Config} */
const colors = require('tailwindcss/colors')

module.exports = {
  content: ["./**/*.{html,hbs}"],
  theme: {
    fontFamily: {
      'sans': ['"Spline Sans Mono"'],
      'mono': ['"Source Code Pro"']
    },
    extend: {
      colors: {
        bg: '#080213',
        card: '#0F131A',
        accent: '#22c55e',
        accent2: '#FFB454',
        link: '#a855f7',
      }
    },
  },
  plugins: [],
}
