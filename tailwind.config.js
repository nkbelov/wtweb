/** @type {import('tailwindcss').Config} */
const colors = require('tailwindcss/colors')

module.exports = {
  content: ["./**/*.{html,hbs}"],
  theme: {
    extend: {
      colors: {
        bg: '#0B0E14',
        card: '#0F131A',
        accent: '#59C2FF',
        accent2: '#FFB454',
        link: '#7FD962',
      }
    },
  },
  plugins: [],
}
