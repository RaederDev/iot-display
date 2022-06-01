module.exports = {
    content: [
        './pages/**/*.{js,ts,jsx,tsx}',
        './components/**/*.{js,ts,jsx,tsx}',
    ],
    theme: {
        fontFamily: {
            sans: ['Roboto', 'sans-serif'],
            dseg: ['DSEG14-Classic', 'sans-serif']
        },
        extend: {
            colors: {
                purple: '#5E4AE3',
                brown: '#6B5E62',
                lightgray: '#EEEEEE',
                blue: '#05668D',
                darkgray: '#303030',
                neongreen: '#59FF59'
            },
        },
    },
    plugins: [],
}
