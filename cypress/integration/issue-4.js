function check() {

}

describe('issue-4', () => {
    it('All buttons are the same color', () => {
        // THIS IS BROKEN SO BAD
        // https://github.com/aDotInTheVoid/xmark/issues/8#issuecomment-745341557
        // https://stackoverflow.com/a/21872327/11466826

        cy.viewport('iphone-6');
        cy.visit('/book-2/format/config/');
        var color;
        cy.get('.mobile-nav-chapters.next i').then(next => {
            // Save the color of the next button
            color = getComputedStyle(next[0]).color;
            // Go forward
            cy.get('.mobile-nav-chapters.next i').click();
            cy.get('.mobile-nav-chapters i').then(x => {
                // Check both buttons are the same color
                expect(x.length).to.equal(2);
                expect(getComputedStyle(x[0]).color).to.equal(color);
                expect(getComputedStyle(x[1]).color).to.equal(color);

                // Go back
                cy.get('.mobile-nav-chapters.previous i').click();
            });
        });

        cy.get('.mobile-nav-chapters i').then(x => {
            // Check both buttons are the same color
            expect(x.length).to.equal(2);
            expect(getComputedStyle(x[0]).color).to.equal(color);
            expect(getComputedStyle(x[1]).color).to.equal(color);
        });


    });

});