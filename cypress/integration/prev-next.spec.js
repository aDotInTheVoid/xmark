/// <reference types="cypress" />

describe("The previous-next buttons", () => {
    it("Only shows a forward button on the first page", () => {
        cy.visit("/book-2/");
        let as = cy.get(".nav-wrapper").children("a");
        as.should('have.length', 1);
        as.click();
        cy.location("pathname").should("eq", "/book-2/cli/");
        cy.get(".nav-wrapper").children("a").should('have.length', 2);
    });

    it("Only shows a backward button on the last page", () => {
        
    })
})