/// <reference types="cypress" />

describe('redirects from base dirs', () => {
    it('/book-1/ -> /book-1/title-page', () =>  {
        cy.visit('/book-1/');
        cy.location('pathname').should('eq', '/book-1/title-page/')
    });

    it('book 2 unefected', () => {
        cy.visit('/book-2/');
        cy.location('pathname').should('eq', '/book-2/');
    });

    it('/book-3/ -> /book-3/pre1/', () => {
        cy.visit('/book-3/');
        cy.location('pathname').should('eq', '/book-3/pre1/');
    })
})